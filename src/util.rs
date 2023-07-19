// pub fn sma(source: crate::Source, length: usize) -> f64 {
//     if source[0..length].len() == 0 {
//         return f64::NAN;
//     }
//     source.iter().take(length).sum::<f64>() / length as f64
// }

// pub fn ema(source: crate::Source, length: usize) -> f64 {
//     if source[0..length].len() == 0 {
//         return f64::NAN;
//     }
//     let alpha = 2.0 / (length as f64 + 1.0);
//     let mut sum = 0.0;
//     for i in 0..source.len() {
//         if i < length {
//             sum += source[i];
//         } else {
//             sum = alpha * source[i] + (1.0 - alpha) * sum;
//         }
//     }
//     sum / length as f64
// }
