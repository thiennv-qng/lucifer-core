///
/// Pool operation trait
///
pub trait F64Trait {
  fn valid(&self) -> Option<bool>;
  fn checked_add(&self, num: f64) -> Option<f64>;
  fn checked_sub(&self, num: f64) -> Option<f64>;
  fn checked_div(&self, num: f64) -> Option<f64>;
  fn checked_mul(&self, num: f64) -> Option<f64>;
  fn checked_pow(&self, num: f64) -> Option<f64>;
}

///
/// Operation trait
///
impl F64Trait for f64 {
  fn valid(&self) -> Option<bool> {
    if !self.is_finite() || self.is_subnormal() {
      return None;
    }
    Some(true)
  }
  fn checked_add(&self, num: f64) -> Option<f64> {
    // Valid params
    self.valid()?;
    num.valid()?;
    // Calculate
    let result = self + num;
    // Valid result
    result.valid()?;
    return Some(result);
  }
  fn checked_sub(&self, num: f64) -> Option<f64> {
    // Valid params
    self.valid()?;
    num.valid()?;
    // Calculate
    let result = self - num;
    // Valid result
    result.valid()?;
    return Some(result);
  }
  fn checked_div(&self, num: f64) -> Option<f64> {
    // Valid params
    self.valid()?;
    num.valid()?;
    // Calculate
    let result = self / num;
    // Valid result
    result.valid()?;
    return Some(result);
  }
  fn checked_mul(&self, num: f64) -> Option<f64> {
    // Valid params
    self.valid()?;
    num.valid()?;
    // Calculate
    let result = self * num;
    // Valid result
    result.valid()?;
    return Some(result);
  }

  fn checked_pow(&self, num: f64) -> Option<f64> {
    // Valid params
    self.valid()?;
    num.valid()?;
    // Calculate
    let result = self.powf(num);
    // Valid result
    result.valid()?;
    return Some(result);
  }
}
