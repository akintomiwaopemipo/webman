use std::path::PathBuf;

use util::unique_characters_from_fs;

pub struct App;


impl App {

    pub fn document_root()->String{
        let mut _dirname = util::current_dir();

        loop{
            if util::file_exists(&format!(r#"{}/.webman/config.db"#,&_dirname)){
                return _dirname;
            }else{
                _dirname = util::dirname(&_dirname);
            }

            if _dirname == "/"{
                return String::from(""); 
            }
        }
    }


    pub fn tmp_directory() -> String{
        let _tmp_directory = PathBuf::from_iter([
            &Self::document_root(),
            "tmp"
        ]).into_os_string().into_string().unwrap();
        if !util::directory_exists(&_tmp_directory){
            Self::mkdir(&_tmp_directory);
        }
        String::from(_tmp_directory)
    }


    pub fn new_tmp_file(file_extension: &str, length: usize) -> String{
        
        let mut extension = format!("{}", file_extension);

        if !extension.is_empty(){
            extension = format!(".{}", file_extension);
        }
    
        let _tmp_directory = Self::tmp_directory();
    
        let file_name = format!("{}", unique_characters_from_fs(&_tmp_directory, length.try_into().unwrap(), Some(&extension)));
    
        format!("{}/{}", _tmp_directory, file_name)
    
    
    }



    /// Make directory recursively
    pub fn mkdir(path: &str){
        std::fs::create_dir_all(path).expect("Error occured while creating directory");
    }


    pub fn copy_directory(from: &str, to: &str){
        Self::mkdir(from);
        Self::mkdir(to);
        fs_extra::dir::copy(
            from, 
            to, 
            &fs_extra::dir::CopyOptions { ..Default::default() }
        ).unwrap();
    }


    pub fn config_file() -> String{
        let document_root = Self::document_root();
        format!("{document_root}/.webman/config.db")
    }

}