use z3::ast::Int;

pub trait IntExtensions {
    fn abs(&self) -> Self;
}

impl IntExtensions for Int {
    fn abs(&self) -> Self {
        let is_negative = self.lt(0);
        let negated = self.unary_minus();
        // if(self < 0) { -self } else { self }
        is_negative.ite(&negated, self)
    }
}
