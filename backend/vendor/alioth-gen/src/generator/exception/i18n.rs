//! Exception I18n Generator
//!
//! 生成国际化消息模板和翻译文件

use crate::generator::ir::exception::GeneratorException;
use crate::generator::{GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata};
use std::collections::HashMap;

/// 国际化消息生成器
pub struct ExceptionI18nGenerator;

impl ExceptionI18nGenerator {
    /// 生成 i18n 配置文件
    pub fn generate_i18n_config(
        default_locale: &str,
        supported_locales: &[&str],
    ) -> Result<String, GenerateError> {
        let locales: Vec<String> = supported_locales
            .iter()
            .map(|s| format!("  - {}", s))
            .collect();
        Ok(format!(
            "default_locale: {}\nsupported_locales:\n{}",
            default_locale,
            locales.join("\n")
        ))
    }

    /// 生成所有国际化资源文件
    pub fn generate_i18n_resources(
        exceptions: &[GeneratorException],
        default_locale: &str,
        supported_locales: &[&str],
    ) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        // Generate default locale file
        let default_content = Self::generate_locale_file(exceptions, default_locale)?;
        let checksum = format!("{:x}", md5::compute(default_content.as_bytes()));
        files.push(GeneratedFile {
            path: format!("locales/{}.json", default_locale).into(),
            content: default_content.clone(),
            checksum,
        });

        // Generate other locale files (placeholders)
        for locale in supported_locales {
            if *locale != default_locale {
                let content = Self::generate_locale_file(exceptions, locale)?;
                let checksum = format!("{:x}", md5::compute(content.as_bytes()));
                files.push(GeneratedFile {
                    path: format!("locales/{}.json", locale).into(),
                    content: content.clone(),
                    checksum,
                });
            }
        }

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_i18n".to_string(),
                entity_count: exceptions.len(),
                c_file_count,
            },
        })
    }

    /// 生成特定语言环境的错误文件
    fn generate_locale_file(
        exceptions: &[GeneratorException],
        locale: &str,
    ) -> Result<String, GenerateError> {
        let mut messages: HashMap<String, serde_json::Value> = HashMap::new();

        // Add common error messages
        messages.insert(
            "error.generic.title".to_string(),
            serde_json::Value::String(Self::translate("An error occurred", locale).to_string()),
        );
        messages.insert(
            "error.generic.retry".to_string(),
            serde_json::Value::String(Self::translate("Please try again", locale).to_string()),
        );

        // Add exception-specific messages
        for exception in exceptions {
            let key = format!("error.{}", exception.name.snake);

            // Default message
            let default_msg = format!("{} error occurred", exception.name.pascal);
            let message = if let Some(ref i18n) = exception.i18n_message {
                i18n.translations
                    .get(locale)
                    .unwrap_or(&i18n.default_message)
                    .clone()
            } else {
                default_msg
            };

            messages.insert(
                key.clone(),
                serde_json::Value::String(Self::translate(&message, locale).to_string()),
            );

            // Add description
            if let Some(ref desc) = exception.description {
                messages.insert(
                    format!("{}.description", key),
                    serde_json::Value::String(Self::translate(desc, locale).to_string()),
                );
            }

            // Add field descriptions
            for field in &exception.fields {
                let field_key = format!("{}.field.{}", key, field.name.snake);
                let field_desc = field.description.as_deref().unwrap_or(&field.name.raw);
                messages.insert(
                    field_key,
                    serde_json::Value::String(Self::translate(field_desc, locale).to_string()),
                );
            }
        }

        // Sort keys for consistent output
        let sorted: std::collections::BTreeMap<_, _> = messages.into_iter().collect();

        serde_json::to_string_pretty(&sorted)
            .map_err(|e| GenerateError::Template(format!("JSON serialization error: {}", e)))
    }

    /// 简单的翻译模拟（在实际项目中会使用专业的翻译服务）
    fn translate(text: &str, locale: &str) -> String {
        // This is a placeholder - real implementation would use a translation service
        match locale {
            "en" => text.to_string(),
            "zh" | "zh-CN" => {
                // Simple translations for common terms
                text.replace("error occurred", "发生错误")
                    .replace("An error occurred", "发生错误")
                    .replace("Please try again", "请重试")
                    .replace("Validation", "验证")
                    .replace("Not Found", "未找到")
                    .replace("Unauthorized", "未授权")
                    .replace("Forbidden", "禁止访问")
                    .replace("Internal Server Error", "服务器内部错误")
            }
            "ja" => text
                .replace("error occurred", "エラーが発生しました")
                .replace("An error occurred", "エラーが発生しました")
                .replace("Please try again", "もう一度お試しください"),
            "ko" => text
                .replace("error occurred", "오류가 발생했습니다")
                .replace("An error occurred", "오류가 발생했습니다")
                .replace("Please try again", "다시 시도해주세요"),
            _ => text.to_string(),
        }
    }

    /// 生成 Rust i18n 模块
    pub fn generate_rust_i18n_module(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_rust_i18n_content(exceptions)?;

        let files = vec![GeneratedFile {
            path: "src/errors/i18n.rs".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_i18n".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 Rust i18n 内容
    fn generate_rust_i18n_content(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("//! Error I18n Support\n\n");
        output.push_str("use std::collections::HashMap;\n\n");

        // Generate I18nError trait
        output.push_str(
            r#"/// Trait for errors that support internationalization
pub trait I18nError {
    /// Get the message key for this error
    fn message_key(&self) -> &str;
    
    /// Get the default message (English)
    fn default_message(&self) -> String;
    
    /// Format the message with given parameters
    fn format_message(&self, params: &HashMap<String, String>) -> String;
}

"#,
        );

        // Generate I18nManager struct
        output.push_str(
            r#"/// I18n message manager
pub struct I18nManager {
    locale: String,
    messages: HashMap<String, String>,
}

impl I18nManager {
    /// Create a new I18nManager with the given locale
    pub fn new(locale: impl Into<String>) -> Self {
        let locale = locale.into();
        let messages = Self::load_messages(&locale);
        Self { locale, messages }
    }
    
    /// Get a message by key
    pub fn get(&self, key: &str) -> &str {
        self.messages.get(key).map(|s| s.as_str()).unwrap_or(key)
    }
    
    /// Format a message with parameters
    pub fn format(&self, key: &str, params: &HashMap<String, String>) -> String {
        let template = self.get(key);
        let mut result = template.to_string();
        
        for (key, value) in params {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        
        result
    }
    
    fn load_messages(locale: &str) -> HashMap<String, String> {
        // In a real implementation, this would load from file
        let mut messages = HashMap::new();
        
        // Default messages
        messages.insert("error.generic.title".to_string(), "An error occurred".to_string());
        messages.insert("error.generic.retry".to_string(), "Please try again".to_string());
        
"#,
        );

        // Add exception messages
        for exception in exceptions {
            let key = format!("error.{}", exception.name.snake);
            let message = if let Some(ref i18n) = exception.i18n_message {
                &i18n.default_message
            } else {
                &format!("{} error", exception.name.pascal)
            };

            output.push_str(&format!(
                r#"        messages.insert("{}".to_string(), "{}".to_string());
"#,
                key, message
            ));
        }

        output.push_str(
            r#"        
        messages
    }
}

"#,
        );

        // Generate Fluent format support
        output.push_str(
            r#"/// Fluent format support (Mozilla Fluent)
pub mod fluent {
    use std::collections::HashMap;
    
    /// Format a Fluent message
    pub fn format(message: &str, args: &HashMap<String, String>) -> String {
        let mut result = message.to_string();
        
        // Simple placeholder replacement
        // Fluent supports more complex patterns like:
        // - {$fieldName}
        // - {$count ->
        //     [one] One item
        //     *[other] {$count} items
        //   }
        for (key, value) in args {
            result = result.replace(&format!("{{${}}}", key), value);
        }
        
        result
    }
}

"#,
        );

        // Generate ICU format support
        output.push_str(
            r#"/// ICU MessageFormat support
pub mod icu {
    use std::collections::HashMap;
    
    /// Format an ICU message
    pub fn format(message: &str, args: &HashMap<String, String>) -> String {
        let mut result = message.to_string();
        
        // Simple placeholder replacement
        // ICU supports:
        // - {fieldName}
        // - {fieldName, number}
        // - {fieldName, date, short}
        // - {count, plural, one {# item} other {# items}}
        for (key, value) in args {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        
        result
    }
}
"#,
        );

        Ok(output)
    }

    /// 生成 TypeScript i18n 模块
    pub fn generate_typescript_i18n_module(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_typescript_i18n_content(exceptions)?;

        let files = vec![GeneratedFile {
            path: "src/errors/i18n.ts".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_i18n".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 TypeScript i18n 内容
    fn generate_typescript_i18n_content(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("// Error I18n Support\n\n");

        // Generate message keys type
        output.push_str("export type ErrorMessageKey =\n");
        output.push_str("  | 'error.generic.title'\n");
        output.push_str("  | 'error.generic.retry'\n");

        for exception in exceptions {
            let key = format!("error.{}", exception.name.snake);
            output.push_str(&format!("  | '{}'\n", key));
        }

        output.push_str("  ;\n\n");

        // Generate I18n interface
        output.push_str(
            r#"export interface I18nConfig {
  locale: string;
  fallbackLocale: string;
  messages: Record<string, string>;
}

export class I18nManager {
  private locale: string;
  private fallbackLocale: string;
  private messages: Map<string, string>;

  constructor(config: I18nConfig) {
    this.locale = config.locale;
    this.fallbackLocale = config.fallbackLocale;
    this.messages = new Map(Object.entries(config.messages));
  }

  get(key: ErrorMessageKey): string {
    return this.messages.get(key) || key;
  }

  format(key: ErrorMessageKey, params: Record<string, string>): string {
    let template = this.get(key);
    
    for (const [key, value] of Object.entries(params)) {
      template = template.replace(new RegExp(`{${key}}`, 'g'), value);
    }
    
    return template;
  }

  setLocale(locale: string): void {
    this.locale = locale;
    // In a real implementation, this would load new messages
  }
}

"#,
        );

        // Generate default messages
        output.push_str("export const defaultMessages: Record<string, string> = {\n");
        output.push_str("  'error.generic.title': 'An error occurred',\n");
        output.push_str("  'error.generic.retry': 'Please try again',\n");

        for exception in exceptions {
            let key = format!("error.{}", exception.name.snake);
            let message = if let Some(ref i18n) = exception.i18n_message {
                &i18n.default_message
            } else {
                &format!("{} error", exception.name.pascal)
            };

            output.push_str(&format!(
                "  '{}': '{}',\n",
                key,
                message.replace("'", "\\'")
            ));
        }

        output.push_str("};\n\n");

        // Generate ICU format helper
        output.push_str(
            r#"// ICU MessageFormat helper
export function formatIcuMessage(
  message: string,
  values: Record<string, string | number>
): string {
  let result = message;
  
  for (const [key, value] of Object.entries(values)) {
    result = result.replace(new RegExp(`{${key}}`, 'g'), String(value));
  }
  
  return result;
}
"#,
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::exception::{ExceptionName, GeneratorI18nMessage, HttpStatusCode};

    fn create_test_exception(name: &str, message: &str) -> GeneratorException {
        GeneratorException {
            name: ExceptionName::from_raw(name),
            description: Some(format!("Test {} error", name)),
            parent_exceptions: vec![],
            fields: vec![],
            error_code: Some(format!("ERR_{}", name.to_uppercase())),
            error_code_constant: format!("{}_ERROR", name.to_uppercase()),
            http_status: HttpStatusCode::BadRequest,
            is_abstract: false,
            inheritance_depth: 0,
            i18n_message: Some(GeneratorI18nMessage {
                message_key: format!("error.{}", name.to_lowercase()),
                default_message: message.to_string(),
                translations: {
                    let mut map = HashMap::new();
                    map.insert("zh".to_string(), format!("{}错误", name));
                    map
                },
                icu_format: false,
                parameters: vec!["field".to_string()],
            }),
        }
    }

    #[test]
    fn test_generate_locale_file() {
        let exceptions = vec![create_test_exception(
            "Validation",
            "Validation failed for {field}",
        )];

        let output = ExceptionI18nGenerator::generate_locale_file(&exceptions, "en").unwrap();

        assert!(output.contains("error.validation"));
        assert!(output.contains("Validation failed"));
    }

    #[test]
    fn test_generate_i18n_config() {
        let output =
            ExceptionI18nGenerator::generate_i18n_config("en", &["en", "zh", "ja"]).unwrap();

        assert!(output.contains("default_locale"));
        assert!(output.contains("supported_locales"));
    }

    #[test]
    fn test_generate_rust_i18n_content() {
        let exceptions = vec![create_test_exception("Validation", "Validation failed")];

        let output = ExceptionI18nGenerator::generate_rust_i18n_content(&exceptions).unwrap();

        assert!(output.contains("trait I18nError"));
        assert!(output.contains("struct I18nManager"));
    }

    #[test]
    fn test_generate_typescript_i18n_content() {
        let exceptions = vec![create_test_exception("Validation", "Validation failed")];

        let output = ExceptionI18nGenerator::generate_typescript_i18n_content(&exceptions).unwrap();

        assert!(output.contains("ErrorMessageKey"));
        assert!(output.contains("class I18nManager"));
    }
}
