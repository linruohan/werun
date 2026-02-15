/// 计算器插件
///
/// 提供数学计算功能
use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};
use anyhow::Result;

/// 计算器插件
pub struct CalculatorPlugin {
    /// 是否启用
    enabled: bool,
}

impl CalculatorPlugin {
    /// 创建新的计算器插件
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// 计算表达式
    fn evaluate(&self, expression: &str) -> Option<f64> {
        // 简单的表达式求值
        // TODO: 使用更强大的表达式解析库

        // 移除空格
        let expr = expression.replace(' ', "");

        // 尝试直接解析为数字
        if let Ok(num) = expr.parse::<f64>() {
            return Some(num);
        }

        // 简单的四则运算
        self.parse_expression(&expr)
    }

    /// 解析简单表达式
    fn parse_expression(&self, expr: &str) -> Option<f64> {
        // 支持 + - * / 和括号
        // 这是一个简化的实现

        // 处理括号
        if let Some(start) = expr.find('(') {
            if let Some(end) = expr.rfind(')') {
                let inner = &expr[start + 1..end];
                if let Some(inner_result) = self.parse_expression(inner) {
                    let new_expr =
                        format!("{}{}{}", &expr[..start], inner_result, &expr[end + 1..]);
                    return self.parse_expression(&new_expr);
                }
            }
        }

        // 处理加法
        if let Some(pos) = expr.find('+') {
            let left = self.parse_expression(&expr[..pos])?;
            let right = self.parse_expression(&expr[pos + 1..])?;
            return Some(left + right);
        }

        // 处理减法
        if let Some(pos) = expr.rfind('-') {
            if pos > 0 {
                let left = self.parse_expression(&expr[..pos])?;
                let right = self.parse_expression(&expr[pos + 1..])?;
                return Some(left - right);
            }
        }

        // 处理乘法
        if let Some(pos) = expr.find('*') {
            let left = self.parse_expression(&expr[..pos])?;
            let right = self.parse_expression(&expr[pos + 1..])?;
            return Some(left * right);
        }

        // 处理除法
        if let Some(pos) = expr.find('/') {
            let left = self.parse_expression(&expr[..pos])?;
            let right = self.parse_expression(&expr[pos + 1..])?;
            if right != 0.0 {
                return Some(left / right);
            }
        }

        // 尝试解析为数字
        expr.parse::<f64>().ok()
    }

    /// 格式化结果
    fn format_result(&self, value: f64) -> String {
        if value == value.trunc() {
            // 整数
            format!("{:.0}", value)
        } else {
            // 小数，保留适当精度
            format!("{:.6}", value)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    }

    /// 检查是否是数学表达式
    fn is_expression(&self, query: &str) -> bool {
        let expr = query.replace(' ', "");

        // 包含运算符
        expr.contains('+')
            || expr.contains('-')
            || expr.contains('*')
            || expr.contains('/')
            || expr.contains('(')
            || expr.contains(')')
            || expr.parse::<f64>().is_ok()
    }
}

impl Plugin for CalculatorPlugin {
    fn id(&self) -> &str {
        "calculator"
    }

    fn name(&self) -> &str {
        "计算器"
    }

    fn description(&self) -> &str {
        "计算数学表达式"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn initialize(&mut self) -> Result<()> {
        log::info!("初始化计算器插件...");
        Ok(())
    }

    fn search(&self, query: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // 检查是否是数学表达式
        if self.is_expression(query) {
            if let Some(value) = self.evaluate(query) {
                let result_str = self.format_result(value);

                results.push(SearchResult {
                    id: format!("calc:{}", query),
                    title: format!("{} = {}", query, result_str),
                    description: "按 Enter 复制结果".to_string(),
                    icon: None,
                    result_type: ResultType::Calculator,
                    score: 1000, // 计算器结果优先级很高
                    action: ActionData::CopyToClipboard { text: result_str },
                });
            }
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::CopyToClipboard { text } = &result.action {
            // TODO: 复制到剪贴板
            log::info!("复制到剪贴板: {}", text);
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}
