use gl;
use std;
use std::ffi::{CString, CStr};
use resources::{self, Resources};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown shader type for resource {}", name)]
    UnknownShaderType { name: String, message: String },
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoadError { name: String, #[cause] inner: resources::Error },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn id(&self) -> gl_builder::types::GLuint {
        self.id
    }

    pub fn use_it(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(gl, res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..]).map_err(|message| Error::LinkError {
            name: name.into(),
            message,
        })
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };
        for shader in shaders {
            unsafe { gl.AttachShader(program_id, shader.id()); }
        }
        unsafe { gl.LinkProgram(program_id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        };

        if success == 0 {
            let mut error_len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut error_len);
            }
            let error_msg: CString = create_whitespace_cstring_with_len(error_len as usize);
            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    error_len,
                    std::ptr::null_mut(),
                    error_msg.as_ptr() as *mut gl::types::GLchar
                );
            }
            return Err(error_msg.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl.DetachShader(program_id, shader.id()); }
        }

        Ok(Program { gl: gl.clone(), id: program_id })
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn id (&self) -> gl::types::GLuint {
        self.id
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::UnknownShaderType {
                name: name.to_owned(),
                message: "failed to recognize shader extension".to_owned()
            })?;

        let source = res.load_cstring(name)
            .map_err(|e| Error::ResourceLoadError {
                name: name.into(),
                inner: e,
            })?;

        Shader::from_source(gl, &source, shader_kind)
    }

    pub fn from_source(
        gl: &gl::Gl,
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<Shader, Error> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    // pub fn from_vert_source(gl_builder: &gl_builder::Gl, source: &CStr) -> Result<Shader, String> {
    //     Shader::from_source(gl_builder, source, gl_builder::VERTEX_SHADER)
    // }
    //
    // pub fn from_frag_source(gl_builder: &gl_builder::Gl, source: &CStr) -> Result<Shader, String> {
    //     Shader::from_source(gl_builder, source, gl_builder::FRAGMENT_SHADER)
    // }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &CStr,
    kind: gl::types::GLuint
) -> Result<gl::types::GLuint, Error> {
    let shader = unsafe { gl.CreateShader(kind) };

    unsafe {
        gl.ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(shader);
    };

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    };

    if success == 0 {
        let mut error_len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut error_len);
        }
        let error_msg: CString = create_whitespace_cstring_with_len(error_len as usize);
        unsafe {
            gl.GetShaderInfoLog(
                shader,
                error_len,
                std::ptr::null_mut(),
                error_msg.as_ptr() as *mut gl::types::GLchar
            );
        }
        return Err(Error::CompileError {
            name: "shader".to_owned(),
            message: error_msg.to_string_lossy().into_owned(),
        })
    }

    Ok(shader)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}