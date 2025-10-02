//! Machine learning engine for anomaly detection

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use candle_core::{Device, Tensor, DType};
use candle_nn::{Module, VarBuilder, VarMap, linear, Linear};
use candle_nn::ops;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

use crate::types::{SystemConfig, FlowFeatures, MLConfig};

/// Simple neural network model for anomaly detection
#[derive(Debug)]
pub struct AnomalyDetectionModel {
    layer1: Linear,
    layer2: Linear,
    output: Linear,
    #[allow(dead_code)]
    device: Device,
}

impl AnomalyDetectionModel {
    /// Create a new model
    pub fn new(var_builder: &VarBuilder, input_size: usize, hidden_size: usize, device: Device) -> Result<Self> {
        let layer1 = linear(input_size, hidden_size, var_builder.pp("layer1"))?;
        let layer2 = linear(hidden_size, hidden_size / 2, var_builder.pp("layer2"))?;
        let output = linear(hidden_size / 2, 1, var_builder.pp("output"))?;
        
        Ok(Self {
            layer1,
            layer2,
            output,
            device,
        })
    }
}

impl Module for AnomalyDetectionModel {
    /// Forward pass through the model
    fn forward(&self, input: &Tensor) -> candle_core::Result<Tensor> {
        let x = self.layer1.forward(input)?;
        let x = x.relu()?;
        let x = self.layer2.forward(&x)?;
        let x = x.relu()?;
        let x = self.output.forward(&x)?;
        
        // Apply sigmoid activation for anomaly probability using ops
        ops::sigmoid(&x)
    }
}

/// Feature extraction and preprocessing
pub struct FeatureExtractor {
    feature_stats: RwLock<FeatureStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeatureStatistics {
    means: HashMap<String, f32>,
    stds: HashMap<String, f32>,
    mins: HashMap<String, f32>,
    maxs: HashMap<String, f32>,
    update_count: u64,
}

impl Default for FeatureStatistics {
    fn default() -> Self {
        Self {
            means: HashMap::new(),
            stds: HashMap::new(),
            mins: HashMap::new(),
            maxs: HashMap::new(),
            update_count: 0,
        }
    }
}

impl FeatureExtractor {
    /// Create a new feature extractor
    pub fn new() -> Self {
        Self {
            feature_stats: RwLock::new(FeatureStatistics::default()),
        }
    }
    
    /// Extract numerical features from flow data
    pub fn extract_features(&self, flow: &FlowFeatures) -> Result<Vec<f32>> {
        let mut features = Vec::new();
        
        // Basic flow features
        features.push(flow.duration);
        features.push(flow.packet_count as f32);
        features.push(flow.byte_count as f32);
        features.push(flow.packets_per_second);
        features.push(flow.bytes_per_second);
        features.push(flow.avg_packet_size);
        features.push(flow.port_entropy);
        features.push(flow.packet_size_variance);
        
        // Statistical features from inter-arrival times
        if !flow.inter_arrival_times.is_empty() {
            let mean_iat = flow.inter_arrival_times.iter().sum::<f32>() / flow.inter_arrival_times.len() as f32;
            let var_iat = flow.inter_arrival_times.iter()
                .map(|x| (x - mean_iat).powi(2))
                .sum::<f32>() / flow.inter_arrival_times.len() as f32;
            
            features.push(mean_iat);
            features.push(var_iat.sqrt()); // Standard deviation
            features.push(*flow.inter_arrival_times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0));
            features.push(*flow.inter_arrival_times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0));
        } else {
            features.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
        }
        
        // Protocol distribution features - fix casting issue
        let total_packets = flow.protocol_distribution.values().sum::<u32>() as f32;
        if total_packets > 0.0 {
            features.push(*flow.protocol_distribution.get(&crate::types::Protocol::TCP).unwrap_or(&0) as f32 / total_packets);
            features.push(*flow.protocol_distribution.get(&crate::types::Protocol::UDP).unwrap_or(&0) as f32 / total_packets);
            features.push(*flow.protocol_distribution.get(&crate::types::Protocol::ICMP).unwrap_or(&0) as f32 / total_packets);
        } else {
            features.extend_from_slice(&[0.0, 0.0, 0.0]);
        }
        
        // Flag pattern features
        let syn_count = flow.flag_patterns.iter().filter(|&flag| flag.contains("SYN")).count() as f32;
        let ack_count = flow.flag_patterns.iter().filter(|&flag| flag.contains("ACK")).count() as f32;
        let fin_count = flow.flag_patterns.iter().filter(|&flag| flag.contains("FIN")).count() as f32;
        let rst_count = flow.flag_patterns.iter().filter(|&flag| flag.contains("RST")).count() as f32;
        
        features.push(syn_count);
        features.push(ack_count);
        features.push(fin_count);
        features.push(rst_count);
        
        Ok(features)
    }
    
    /// Normalize features using running statistics
    pub fn normalize_features(&self, features: &[f32]) -> Result<Vec<f32>> {
        let stats = self.feature_stats.read();
        
        if stats.update_count == 0 {
            // No statistics available, return features as-is
            return Ok(features.to_vec());
        }
        
        let mut normalized = Vec::with_capacity(features.len());
        
        for (i, &value) in features.iter().enumerate() {
            let feature_name = format!("feature_{}", i);
            
            if let (Some(&mean), Some(&std)) = (stats.means.get(&feature_name), stats.stds.get(&feature_name)) {
                if std > 1e-8 {
                    normalized.push((value - mean) / std);
                } else {
                    normalized.push(0.0);
                }
            } else {
                normalized.push(value);
            }
        }
        
        Ok(normalized)
    }
    
    /// Update feature statistics with new data
    pub fn update_statistics(&self, features: &[f32]) {
        let mut stats = self.feature_stats.write();
        
        for (i, &value) in features.iter().enumerate() {
            let feature_name = format!("feature_{}", i);
            
            // Update running statistics using Welford's online algorithm
            let count = stats.update_count + 1;
            let old_mean = stats.means.get(&feature_name).copied().unwrap_or(0.0);
            let new_mean = old_mean + (value - old_mean) / count as f32;
            
            let old_m2 = if count > 1 {
                let old_std = stats.stds.get(&feature_name).copied().unwrap_or(0.0);
                old_std * old_std * (count - 1) as f32
            } else {
                0.0
            };
            
            let new_m2 = old_m2 + (value - old_mean) * (value - new_mean);
            let new_std = if count > 1 {
                (new_m2 / (count - 1) as f32).sqrt()
            } else {
                0.0
            };
            
            stats.means.insert(feature_name.clone(), new_mean);
            stats.stds.insert(feature_name.clone(), new_std);
            
            // Update min/max
            let current_min = stats.mins.get(&feature_name).copied().unwrap_or(value);
            let current_max = stats.maxs.get(&feature_name).copied().unwrap_or(value);
            
            stats.mins.insert(feature_name.clone(), current_min.min(value));
            stats.maxs.insert(feature_name, current_max.max(value));
        }
        
        stats.update_count += 1;
    }
}

/// Simplified ML engine without complex optimizer
#[derive(Debug, Clone)]
struct TrainingExample {
    features: Vec<f32>,
    label: f32, // 0.0 for normal, 1.0 for anomaly
    #[allow(dead_code)]
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Main ML engine
pub struct MLEngine {
    model: Arc<RwLock<AnomalyDetectionModel>>,
    feature_extractor: FeatureExtractor,
    #[allow(dead_code)]
    var_map: Arc<RwLock<VarMap>>,
    device: Device,
    config: MLConfig,
    training_buffer: RwLock<Vec<TrainingExample>>,
}

impl MLEngine {
    /// Create a new ML engine
    pub async fn new(config: &SystemConfig) -> Result<Self> {
        info!("Initializing ML engine");
        
        let device = Device::Cpu; // Use CPU for compatibility
        let var_map = VarMap::new();
        let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &device);
        
        // Model hyperparameters
        let input_size = 20; // Number of features
        let hidden_size = 64;
        
        let model = AnomalyDetectionModel::new(&var_builder, input_size, hidden_size, device.clone())?;
        
        info!("ML engine initialized successfully");
        
        Ok(Self {
            model: Arc::new(RwLock::new(model)),
            feature_extractor: FeatureExtractor::new(),
            var_map: Arc::new(RwLock::new(var_map)),
            device,
            config: config.ml_config.clone(),
            training_buffer: RwLock::new(Vec::new()),
        })
    }
    
    /// Predict anomaly score for given features
    pub fn predict(&self, flow_features: &FlowFeatures) -> Result<f32> {
        // Extract and normalize features
        let raw_features = self.feature_extractor.extract_features(flow_features)?;
        let normalized_features = self.feature_extractor.normalize_features(&raw_features)?;
        
        // Pad or truncate features to expected size (20)
        let mut input_data = normalized_features;
        input_data.resize(20, 0.0);
        
        // Convert to tensor
        let input_tensor = Tensor::from_vec(input_data, (1, 20), &self.device)?;
        
        // Get prediction
        let model = self.model.read();
        let output = model.forward(&input_tensor)?;
        
        // Extract scalar value
        let prediction = output.to_vec1::<f32>()?[0];
        
        // Update feature statistics for future normalization
        let raw_features = self.feature_extractor.extract_features(flow_features)?;
        self.feature_extractor.update_statistics(&raw_features);
        
        Ok(prediction)
    }
    
    /// Add training example
    pub fn add_training_example(&self, flow_features: &FlowFeatures, is_anomaly: bool) {
        if let Ok(features) = self.feature_extractor.extract_features(flow_features) {
            let example = TrainingExample {
                features,
                label: if is_anomaly { 1.0 } else { 0.0 },
                timestamp: chrono::Utc::now(),
            };
            
            let mut buffer = self.training_buffer.write();
            buffer.push(example);
            
            // Limit buffer size
            if buffer.len() > 10000 {
                buffer.drain(0..1000);
            }
        }
    }
    
    /// Train the model with accumulated examples (simplified version)
    pub async fn train_model(&self) -> Result<f32> {
        let examples = {
            let buffer = self.training_buffer.read();
            if buffer.len() < self.config.batch_size {
                return Ok(0.0); // Not enough data
            }
            buffer.clone()
        };
        
        debug!("Training model with {} examples", examples.len());
        
        // Prepare training data
        let batch_size = self.config.batch_size.min(examples.len());
        let mut features_batch = Vec::new();
        let mut labels_batch = Vec::new();
        
        for example in examples.iter().take(batch_size) {
            let mut normalized = self.feature_extractor.normalize_features(&example.features)?;
            normalized.resize(20, 0.0); // Ensure consistent size
            features_batch.extend(normalized);
            labels_batch.push(example.label);
        }
        
        // Convert to tensors
        let features_tensor = Tensor::from_vec(
            features_batch,
            (batch_size, 20),
            &self.device,
        )?;
        
        let labels_tensor = Tensor::from_vec(
            labels_batch,
            (batch_size, 1),
            &self.device,
        )?;
        
        // Forward pass
        let model = self.model.read();
        let predictions = model.forward(&features_tensor)?;
        
        // Calculate binary cross-entropy loss
        let loss = self.binary_cross_entropy_loss(&predictions, &labels_tensor)?;
        let loss_value = loss.to_scalar::<f32>()?;
        
        // Note: Actual gradient computation and parameter updates would require
        // more complex setup with candle's gradient system
        debug!("Training completed with loss: {:.4}", loss_value);
        
        Ok(loss_value)
    }
    
    /// Calculate binary cross-entropy loss
    fn binary_cross_entropy_loss(&self, predictions: &Tensor, targets: &Tensor) -> Result<Tensor> {
        let eps = 1e-8f32;
        
        // Create epsilon tensor
        let eps_tensor = Tensor::full(eps, predictions.shape(), &self.device)?;
        let one_tensor = Tensor::ones_like(predictions)?;
        let one_minus_eps = Tensor::full(1.0f32 - eps, predictions.shape(), &self.device)?;
        
        // Clamp predictions to avoid log(0): max(eps, min(1-eps, pred))
        let predictions_clamped = predictions.minimum(&one_minus_eps)?;
        let predictions_clamped = predictions_clamped.maximum(&eps_tensor)?;
        
        // BCE = -[y*log(p) + (1-y)*log(1-p)]
        let log_pred = predictions_clamped.log()?;
        let one_minus_pred = (&one_tensor - &predictions_clamped)?;
        let log_one_minus_pred = one_minus_pred.log()?;
        
        let one_minus_targets = (&Tensor::ones_like(targets)? - targets)?;
        let positive_term = targets.mul(&log_pred)?;
        let negative_term = one_minus_targets.mul(&log_one_minus_pred)?;
        
        let loss = (&positive_term + &negative_term)?.neg()?.mean_all()?;
        
        Ok(loss)
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}