use color_eyre::{Result, eyre::eyre};

/// 商品コード
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductCode(String);

impl ProductCode {
    pub fn new(code: String) -> Result<Self> {
        if code.trim().is_empty() {
            return Err(eyre!("商品コードが空です"));
        }
        Ok(Self(code.trim().to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// 数量
#[derive(Debug, Clone, Copy)]
pub struct Quantity(f64);

impl Quantity {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(eyre!("数量が負の値です: {}", value));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// 金額（小計）
#[derive(Debug, Clone, Copy)]
pub struct Amount(f64);

impl Amount {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(eyre!("金額が負の値です: {}", value));
        }
        Ok(Self(value))
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn add(&self, other: &Amount) -> Amount {
        Amount(self.0 + other.0)
    }

    pub fn multiply(&self, ratio: f64) -> Amount {
        Amount(self.0 * ratio)
    }

    pub fn divide_by(&self, divisor: f64) -> Amount {
        Amount(self.0 / divisor)
    }
}

/// 歩留率
#[derive(Debug, Clone, Copy)]
pub struct YieldRate(f64);

impl YieldRate {
    pub fn new(value: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&value) {
            return Err(eyre!(
                "歩留率は0.0から1.0の範囲である必要があります: {}",
                value
            ));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// 消費比率
#[derive(Debug, Clone, Copy)]
pub struct ConsumptionRatio(f64);

impl ConsumptionRatio {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(eyre!("消費比率が負の値です: {}", value));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}
