
#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Value {
    Null,
    Number(i64),
    Float(f32),
    Double(f64),
    Str(String),
    Object(String),
    Char(char),
    Boolean(bool)
}

impl std::ops::Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        if matches!(self, Value::Number(_)) && matches!(rhs, Value::Number(_)) {
            Value::Number(self.to_literal()+rhs.to_literal())
        } else if matches!(self, Value::Str(_)) && matches!(rhs, Value::Str(_)) {
            Value::Str(format!("{}{}", self.to_string(),rhs.to_string()))
        } else if matches!(self, Value::Float(_)) || matches!(rhs, Value::Float(_)) {
            Value::Float(self.to_float()+rhs.to_float())
        }
        else {
            Value::Null
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        if matches!(self, Value::Number(_)) {
            Value::Number(-self.to_literal())
        } else {
            panic!("[NEG]: rhs MUST be integer or float.");
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        if matches!(self, Value::Number(_)) && matches!(rhs, Value::Number(_)) {
            Value::Number(self.to_literal()-rhs.to_literal())
        }
        else {
            panic!("[SUB] Both value MUST Be integer or float.");
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        if matches!(self, Value::Number(_)) && matches!(rhs, Value::Number(_)) {
            Value::Number(self.to_literal()*rhs.to_literal())
        }
        else {
            panic!("[MUL] Both value MUST Be integer or float.");
        }
    }
}

impl std::ops::Not for Value {
    type Output = Value;
    fn not(self) -> Self::Output {
        if matches!(self, Value::Number(_)) {
            if self.clone().to_literal() == 0 {
                Value::Number(1)
            } else if self.clone().to_literal() == 1 {
                Value::Number(0)
            } else {
                Value::Number(!self.to_literal())
            }
        } else {
            panic!("[NOT] It can only apply to i64, not string nor float.");
        }
    }
}

impl std::ops::Shl for Value {
    type Output = Value;
    fn shl(self, rhs: Self) -> Self::Output {
        if matches!(self, Value::Number(_)) && matches!(rhs, Value::Number(_)) {
            if self.clone().to_literal() >= 0 && rhs.clone().to_literal() >= 0 {
                Value::Number(self.to_literal() << rhs.to_literal())
            } else {
                panic!("[SHL] It can only be applied for natural number")
            }
        } else {
            panic!("[SHL] It can only be applied for natural number")
        }
    }
}

impl std::ops::Shr for Value {
    type Output = Value;
    fn shr(self, rhs: Self) -> Self::Output {
        if matches!(self, Value::Number(_)) && matches!(rhs, Value::Number(_)) {
            if self.clone().to_literal() >= 0 && rhs.clone().to_literal() >= 0 {
                Value::Number(self.to_literal() >> rhs.to_literal())
            } else {
                panic!("[SHR] It can only be applied for natural number")
            }
        } else {
            panic!("[SHR] It can only be applied for natural number")
        }
    }
}

impl std::ops::Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        if matches!(self, Value::Number(_)) && matches!(rhs, Value::Number(_)) {
            if self.clone().to_literal() == 0 || rhs.clone().to_literal() == 0 {
                panic!("[DIV] Division by 0");
            }
            Value::Number(self.to_literal()/rhs.to_literal())
        }
        else {
            panic!("[DIV] Both value MUST Be integer or float.");
        }
    }
}

macro_rules! get_cast_value {
    ($self: ident, $data_type: ident) => {
        $self.to_any().downcast_ref::<$data_type>().unwrap()
    };
}

impl Value {
    pub fn new_obj(obj_name: String) -> Self {
        Self::Object(obj_name)
    }
    pub fn new_boolean_from(v: i64) -> Self {
        if v == 0 { Self::Boolean(false) }
        else { Self::Boolean(true) }
    }
    pub fn new(string: String) -> Self {
        // convert string to specified value
        let strtrim = string.trim();
        if strtrim.parse::<i64>().is_ok() {
            Self::Number(strtrim.parse::<i64>().unwrap())
        } else if strtrim.parse::<f32>().is_ok() {
            Self::Float(strtrim.parse::<f32>().unwrap())
        } else if strtrim.parse::<f64>().is_ok() {
            Self::Double(strtrim.parse::<f64>().unwrap())
        } else if strtrim.parse::<bool>().is_ok() {
            Self::Boolean(strtrim.parse::<bool>().unwrap())
        } else if strtrim.parse::<char>().is_ok() {
            Self::Char(strtrim.parse::<char>().unwrap())
        } else {
            if strtrim.len() == 0 {
                Self::Null
            } else {
                Self::Str(strtrim.to_string())
            }
        }
    }

    pub fn to_any(self) -> Box<dyn std::any::Any> {
        self.value().expect("null value")
    }

    pub fn to_literal(self) -> i64 {
        //*self.value().expect("value").downcast_ref::<i64>().unwrap()
        *get_cast_value!(self, i64)
    }

    pub fn to_float(self) -> f32 {
        //*self.value().expect("null value").downcast_ref::<f32>().unwrap()
        *get_cast_value!(self, f32)
    }

    pub fn to_char(self) -> char {
        //*self.value().expect("null value").downcast_ref()
        *get_cast_value!(self, char)
    }

    pub fn to_double(self) -> f64 {
        *get_cast_value!(self, f64)
    }

    pub fn to_string(self) -> String {
        //self.value().expect("null value").downcast_ref::<String>().unwrap().clone()
        get_cast_value!(self,String).clone()
    }

    pub fn value(&self) -> Option<Box<dyn std::any::Any>> {
        match self {
            Value::Null => None,
            Value::Number(number) => Some(Box::new(number.clone())),
            Value::Float(float) => Some(Box::new(float.clone())),
            Value::Double(double) => Some(Box::new(double.clone())),
            Value::Str(string) => Some(Box::new(string[1..string.len()-1].to_string())),
            Value::Object(obj) => Some(Box::new(obj.clone())),
            Value::Boolean(b) => Some(Box::new(b.clone())),
            Value::Char(c) => Some(Box::new(c.clone()))
        }
    }
}
