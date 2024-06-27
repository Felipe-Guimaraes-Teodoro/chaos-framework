#![allow(deref_nullptr)]

#[macro_export]
macro_rules! gen_attrib_pointers {
    ($struct_name:ident, $($index:expr => $field_name:ident: $dimension:expr),*) => {
        $(
            let offset = &((*std::ptr::null::<$struct_name>()).$field_name) as *const _ as *const std::ffi::c_void;
            EnableVertexAttribArray($index);
            VertexAttribPointer($index, $dimension, FLOAT, FALSE, std::mem::size_of::<$struct_name>() as GLsizei, offset);
        )*
    };
}

#[macro_export]
macro_rules! bind_buffer {
    ($buffer_type:expr, $buffer:expr, $data:expr) => {{
        BindBuffer($buffer_type, $buffer);
        let size = ($data.len() * std::mem::size_of_val(&$data[0])) as isize;
        let data_ptr = &$data[0] as *const _ as *const std::ffi::c_void;
        BufferData($buffer_type, size, data_ptr, STATIC_DRAW);
    }};
}
