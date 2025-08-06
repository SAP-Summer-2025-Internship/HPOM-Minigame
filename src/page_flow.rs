#[derive(Debug, Clone)]
pub struct PageFlowValidator {
    button_presses: Vec<String>,
    current_page: usize,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidButton(String, Vec<String>), // button, allowed_buttons
    InvalidPage(usize),
    NoTransitionDefined(usize),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

impl PageFlowValidator {
    pub fn new() -> Self {
        PageFlowValidator { 
            button_presses: Vec::new(),
            current_page: 1,
        }
    }
    
    /// Process a button press and return the next page if valid
    /// HARDCODED FLOW LOGIC:
    /// 1. Page 1: "start" -> Page 2
    /// 2. Page 2: "pm"|"ux"|"engi"|"dm" -> Page 3
    /// 3. Page 3: "mc" -> Page 4, "tf" -> Page 5
    /// 4. Page 4: "4a"|"4b"|"4c"|"4d" -> Page 6 (only if "mc" was pressed)
    /// 5. Page 5: "5t"|"5f" -> Page 7 (only if "tf" was pressed)
    /// 6. Page 6: "6a"|"6b"|"6c"|"6d" -> Page 8 (only if "mc" was pressed)
    /// 7. Page 7: "7t"|"7f" -> Page 8 (only if "tf" was pressed)
    /// 8. Page 8: "trophy" -> Page 9
    /// 9. Page 9: No button presses allowed
    pub fn process_button_press(&mut self, button: &str) -> ValidationResult<usize> {
        let next_page = match self.current_page {
            1 => {
                // Page 1: Only "start" allowed
                if button == "start" {
                    Ok(2)
                } else {
                    Err(ValidationError::InvalidButton(
                        button.to_string(), 
                        vec!["start".to_string()]
                    ))
                }
            },
            
            2 => {
                // Page 2: Role selection
                match button {
                    "pm" | "ux" | "engi" | "dm" => Ok(3),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["pm".to_string(), "ux".to_string(), "engi".to_string(), "dm".to_string()]
                    ))
                }
            },
            
            3 => {
                // Page 3: Multiple choice or True/False selection
                match button {
                    "mc" => Ok(4),
                    "tf" => Ok(5),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["mc".to_string(), "tf".to_string()]
                    ))
                }
            },
            
            4 => {
                // Page 4: Multiple choice questions (only if "mc" was pressed)
                if !self.button_presses.contains(&"mc".to_string()) {
                    return Err(ValidationError::InvalidPage(4));
                }
                match button {
                    "4a" | "4b" | "4c" | "4d" => Ok(6),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["4a".to_string(), "4b".to_string(), "4c".to_string(), "4d".to_string()]
                    ))
                }
            },
            
            5 => {
                // Page 5: True/False questions (only if "tf" was pressed)
                if !self.button_presses.contains(&"tf".to_string()) {
                    return Err(ValidationError::InvalidPage(5));
                }
                match button {
                    "5t" | "5f" => Ok(7),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["5t".to_string(), "5f".to_string()]
                    ))
                }
            },
            
            6 => {
                // Page 6: More multiple choice questions (only if "mc" was pressed)
                if !self.button_presses.contains(&"mc".to_string()) {
                    return Err(ValidationError::InvalidPage(6));
                }
                match button {
                    "6a" | "6b" | "6c" | "6d" => Ok(8),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["6a".to_string(), "6b".to_string(), "6c".to_string(), "6d".to_string()]
                    ))
                }
            },
            
            7 => {
                // Page 7: More true/false questions (only if "tf" was pressed)
                if !self.button_presses.contains(&"tf".to_string()) {
                    return Err(ValidationError::InvalidPage(7));
                }
                match button {
                    "7t" | "7f" => Ok(8),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["7t".to_string(), "7f".to_string()]
                    ))
                }
            },
            
            8 => {
                // Page 8: Trophy button
                if button == "trophy" {
                    Ok(9)
                } else {
                    Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["trophy".to_string()]
                    ))
                }
            },
            
            9 => {
                // Page 9: No button presses allowed
                Err(ValidationError::InvalidButton(
                    button.to_string(),
                    vec![] // No buttons allowed
                ))
            },
            
            _ => Err(ValidationError::NoTransitionDefined(self.current_page))
        }?;
        
        // If we get here, the button press was valid
        self.button_presses.push(button.to_string());
        self.current_page = next_page;
        
        Ok(next_page)
    }
    
    /// Get current page
    pub fn current_page(&self) -> usize {
        self.current_page
    }
    
    /// Get all button presses
    pub fn button_presses(&self) -> &[String] {
        &self.button_presses
    }
    
    // /// Get allowed buttons for current page
    // pub fn allowed_buttons(&self) -> ValidationResult<Vec<String>> {
    //     match self.current_page {
    //         1 => Ok(vec!["start".to_string()]),
    //         2 => Ok(vec!["pm".to_string(), "ux".to_string(), "engi".to_string(), "dm".to_string()]),
    //         3 => Ok(vec!["mc".to_string(), "tf".to_string()]),
    //         4 => {
    //             if self.button_presses.contains(&"mc".to_string()) {
    //                 Ok(vec!["4a".to_string(), "4b".to_string(), "4c".to_string(), "4d".to_string()])
    //             } else {
    //                 Err(ValidationError::InvalidPage(4))
    //             }
    //         },
    //         5 => {
    //             if self.button_presses.contains(&"tf".to_string()) {
    //                 Ok(vec!["5t".to_string(), "5f".to_string()])
    //             } else {
    //                 Err(ValidationError::InvalidPage(5))
    //             }
    //         },
    //         6 => {
    //             if self.button_presses.contains(&"mc".to_string()) {
    //                 Ok(vec!["6a".to_string(), "6b".to_string(), "6c".to_string(), "6d".to_string()])
    //             } else {
    //                 Err(ValidationError::InvalidPage(6))
    //             }
    //         },
    //         7 => {
    //             if self.button_presses.contains(&"tf".to_string()) {
    //                 Ok(vec!["7t".to_string(), "7f".to_string()])
    //             } else {
    //                 Err(ValidationError::InvalidPage(7))
    //             }
    //         },
    //         8 => Ok(vec!["trophy".to_string()]),
    //         9 => Ok(vec![]), // No buttons allowed on final page
    //         _ => Err(ValidationError::NoTransitionDefined(self.current_page))
    //     }
    // }
    
    // /// Check if a button is valid for current page (without processing)
    // pub fn is_button_valid(&self, button: &str) -> bool {
    //     self.allowed_buttons()
    //         .map(|buttons| buttons.contains(&button.to_string()))
    //         .unwrap_or(false)
    // }
    
    // /// Reset the flow validator to initial state
    // pub fn reset(&mut self) {
    //     self.button_presses.clear();
    //     self.current_page = 1;
    // }
    
    // /// Check if a page exists in the flow
    // pub fn is_valid_page(&self, page: usize) -> bool {
    //     match page {
    //         1..=9 => true,
    //         _ => false,
    //     }
    // }
}
