use std::cmp::Ordering;

pub struct ColumnStats {
    pub name: String,
    pub values: Vec<f64>,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
    pub count: u64,
}

impl ColumnStats {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: Vec::new(),
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            count: 0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.values.push(value);
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1;
    }

    /// Финальные вычисления: mean, std, percentiles
    pub fn finalize(&mut self) -> FinalStats {
        if self.count == 0 {
            return FinalStats::empty(&self.name);
        }

        let mean = self.sum / self.count as f64;

        // стандартное отклонение
        let std = if self.values.len() > 1 {
            let variance = self.values.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / (self.values.len() as f64 - 1.0);

            variance.sqrt()
        } else {
            0.0
        };

        // сортировка для перцентилей
        self.values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let q = |p: f64| -> f64 {
            let idx = (p * self.values.len() as f64).floor() as usize;
            self.values[idx.min(self.values.len() - 1)]
        };

        FinalStats {
            name: self.name.clone(),
            count: self.count,
            mean,
            std,
            min: self.min,
            p25: q(0.25),
            p50: q(0.50),
            p75: q(0.75),
            max: self.max,
        }
    }
}

pub struct FinalStats {
    pub name: String,
    pub count: u64,
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub max: f64,
}

impl FinalStats {
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            count: 0,
            mean: 0.0,
            std: 0.0,
            min: 0.0,
            p25: 0.0,
            p50: 0.0,
            p75: 0.0,
            max: 0.0,
        }
    }
}