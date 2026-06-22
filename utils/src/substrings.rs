use indexmap::{ IndexMap, indexmap };
use regex::Regex;

#[derive(Clone)]
pub struct Substrings{
    pub substring_chars: Vec<String>,
    pub substrings: Vec<String>
}


impl Substrings{
    pub fn new(string: &str) -> Self{

        let mut substrings: Vec<String> = vec![];
        let mut substring_chars: Vec<String> = vec![];
        let mut current_substring = "".to_string(); 

        let re = Regex::new(r#"\w|'"#).unwrap(); 

        for (index, char) in string.chars().into_iter().enumerate(){
            if let Some(next_char) = string.chars().into_iter().nth(index+1){
                
                if !char.to_string().trim().is_empty(){
                    if re.is_match(&char.to_string()){
                        current_substring = format!("{current_substring}{char}");
                        
                        if next_char.to_string().trim().is_empty(){
                            
                            if !current_substring.is_empty(){
                                substring_chars.push(current_substring.clone());
                                substrings.push(current_substring.clone());
                            }
                            
                            current_substring = "".to_string();
                        }
                    }else{
                        if !re.is_match(&next_char.to_string()){
                            if !current_substring.is_empty(){
                                substring_chars.push(current_substring.clone());
                                substrings.push(current_substring.clone());
                            }
                        
                            current_substring = char.to_string();
                            
                            if !current_substring.is_empty(){
                                substring_chars.push(current_substring.clone());
                                substrings.push(current_substring.clone());
                            }
                            
                            current_substring = "".to_string();
                        }else{
                            current_substring = format!("{current_substring}{char}");
                        }
                        
                        
                    }
                    
                }else{
                    if !char.to_string().is_empty(){
                        substring_chars.push(format!("{char}"));
                    }
                }

            }else{
                if !char.to_string().trim().is_empty(){
                    if re.is_match(&char.to_string()){
                        
                        if !current_substring.is_empty(){
                            current_substring = format!("{current_substring}{char}");
                            substring_chars.push(current_substring.clone());
                        }
                        
                        substrings.push(current_substring.clone())
                    }else{
                        
                        if !current_substring.is_empty(){
                            substring_chars.push(current_substring.clone());
                            substrings.push(current_substring.clone());
                        }

                        current_substring = char.to_string();
                        
                        if !current_substring.is_empty(){
                            substring_chars.push(current_substring.clone());
                            substrings.push(current_substring.clone());
                        }
                        
                        current_substring = "".to_string();

                    }
                }
            }
        }


        Substrings{
            substrings,
            substring_chars
        }
    }


    pub fn set(&mut self, index: usize, value: &str) -> &mut Self{
        let search_index= index;
        let replace_term = value;

        let mut real_index_opt: Option<usize> = None;
        for (index, substring_char) in self.substring_chars.clone().into_iter().enumerate(){
            if !substring_char.trim().is_empty(){
                if let Some(real_index) = real_index_opt{
                    real_index_opt = Some(real_index + 1);
                }else{
                    real_index_opt = Some(0);
                }

                if search_index == real_index_opt.unwrap(){
                    self.substring_chars[index] = replace_term.to_string();
                    self.substrings[search_index] = replace_term.to_string();
                }
            }
        }

        self
    }


    pub fn update<F>(&mut self, update_func: F) -> &mut Self
        where
            F: Fn(String) -> String
    {
        for (index, substring) in self.clone().into_iter().enumerate(){
            self.set(index, &update_func(substring));
        }
        self
    }



    pub fn get(&self, index: usize) -> String{
        self.substrings[index].clone()
    }


    pub fn find(&self, search: &str) -> Vec<usize>{
        let mut results = vec![];

        for (index, substring) in self.substrings.clone().into_iter().enumerate(){
            if substring == search{
                results.push(index);
            }
        }

        results
    }



    pub fn replace(&mut self, map: IndexMap<usize, String>) -> &mut Self{
        for (index, value) in map{
            self.set(index, &value);
        }
        self
    }


    pub fn find_and_replace<T>(&mut self, search: &str, replace: Vec<T>) -> &mut Self
        where
            T: Into<String> + Clone
    {
        let mut map: IndexMap<usize, String> = indexmap!{};

        for (search_index,index) in self.find(search).into_iter().enumerate(){
            map.insert(index, replace[search_index].clone().into());
        }

        self.replace(map);

        self
    }



    pub fn to_string(&self) -> String{
        self.substring_chars.clone().into_iter().collect()
    }


}



impl IntoIterator for Substrings {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.substrings.into_iter()
    }
}