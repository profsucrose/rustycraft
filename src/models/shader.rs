use std::fs;
use std::ffi::CString;
use gl::types::*;
use std::ptr;
use std::str;

pub struct Shader {
    id: u32
}

impl Shader {
    pub unsafe fn new(vertex_path: &str, fragment_path: &str) -> Shader {
        let vertex_source = fs::read_to_string(vertex_path)
            .expect(format!("Could not read vertex shader at path {}", vertex_path).as_str());
        let fragment_source = fs::read_to_string(fragment_path)
            .expect(format!("Could not read fragment shader at path {}", fragment_path).as_str());
        let vertex_source_cstring = CString::new(vertex_source.as_bytes())
            .expect("Could not convert vertex shader source to CString");
        let fragment_source_cstring = CString::new(fragment_source.as_bytes())
            .expect("Could not convert fragment shader source to CString");    

        // compile vertex shader and check for errors
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &CString::new(vertex_source_cstring.as_bytes()).unwrap().as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip trailing null char
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(vertex_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("VERTEX SHADER COMPILATION FAILED\n{:?}", str::from_utf8(&info_log).unwrap());
        }

        // compile fragment shader and check for errors
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &CString::new(fragment_source_cstring.as_bytes()).unwrap().as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip trailing null char
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("FRAGMENT SHADER COMPILATION FAILED\n{:?}", str::from_utf8(&info_log).unwrap());
        }

        // shader program
        let id = gl::CreateProgram();
        gl::AttachShader(id, vertex_shader);
        gl::AttachShader(id, fragment_shader);
        gl::LinkProgram(id);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip trailing null char
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(id, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("SHADER PROGRAM LINKING FAILED\n{:?}", str::from_utf8(&info_log).unwrap());
        }

        // delete shaders as they're no longer needed
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        Shader { id }
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    // uniforms
    pub unsafe fn set_bool(&self, name: &str, value: bool) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr()), value as i32)
    }

    pub unsafe fn set_int(&self, name: &str, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr()), value)
    }

    pub unsafe fn set_float(&self, name: &str, value: f32) {
        gl::Uniform1f(gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr()), value)
    }
}