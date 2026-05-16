#[macro_export]
macro_rules! prop_color {
    ($ctx:expr, $group:expr, $name:expr, $default:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_color($group, $name, $default);
        reg.color($name)
    }};
}

#[macro_export]
macro_rules! prop_float {
    ($ctx:expr, $group:expr, $name:expr, $default:expr, $min:expr, $max:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_float($group, $name, $default, $min, $max);
        reg.float($name)
    }};
}

#[macro_export]
macro_rules! prop_bool {
    ($ctx:expr, $group:expr, $name:expr, $default:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_bool($group, $name, $default);
        reg.bool_val($name)
    }};
}

#[macro_export]
macro_rules! prop_rounding {
    ($ctx:expr, $group:expr, $name:expr, $default:expr, $min:expr, $max:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_rounding($group, $name, $default, $min, $max);
        reg.rounding($name)
    }};
}

#[macro_export]
macro_rules! prop_vec2 {
    ($ctx:expr, $group:expr, $name:expr, $default:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_vec2($group, $name, $default);
        reg.vec2($name)
    }};
}
