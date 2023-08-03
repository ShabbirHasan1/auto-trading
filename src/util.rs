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

pub fn time_to_string(time: u64) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(
        chrono::NaiveDateTime::from_timestamp_millis(time as i64).unwrap(),
        chrono::Utc,
    );
    let local_datetime: chrono::DateTime<chrono::Local> = datetime.into();
    local_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
