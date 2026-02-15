/// 模糊搜索工具
///
/// 提供高性能的模糊匹配功能

/// 计算模糊匹配分数
///
/// 返回 (是否匹配, 匹配分数)
/// 分数越高表示匹配度越好
pub fn fuzzy_match(query: &str, target: &str) -> (bool, u32) {
    let query = query.to_lowercase();
    let target = target.to_lowercase();

    // 空查询匹配所有
    if query.is_empty() {
        return (true, 0);
    }

    // 精确包含匹配
    if target.contains(&query) {
        let score = calculate_contain_score(&query, &target);
        return (true, score);
    }

    // 字符顺序匹配
    if fuzzy_char_match(&query, &target) {
        let score = calculate_fuzzy_score(&query, &target);
        return (true, score);
    }

    (false, 0)
}

/// 计算包含匹配的分数
fn calculate_contain_score(query: &str, target: &str) -> u32 {
    let mut score = 100u32;

    // 开头匹配加分
    if target.starts_with(query) {
        score += 50;
    }

    // 单词边界匹配加分
    if target.contains(&format!(" {}", query)) {
        score += 30;
    }

    // 长度差异惩罚
    let length_diff = target.len() as i32 - query.len() as i32;
    score -= (length_diff * 2) as u32;

    score
}

/// 字符顺序匹配
///
/// 检查 query 中的字符是否按顺序出现在 target 中
fn fuzzy_char_match(query: &str, target: &str) -> bool {
    let mut query_chars = query.chars();
    let mut current_char = query_chars.next();

    for target_char in target.chars() {
        if let Some(qc) = current_char {
            if target_char == qc {
                current_char = query_chars.next();
            }
        } else {
            // 所有字符都匹配了
            return true;
        }
    }

    // 检查是否所有字符都匹配
    current_char.is_none()
}

/// 计算模糊匹配分数
fn calculate_fuzzy_score(query: &str, target: &str) -> u32 {
    let mut score = 50u32;

    // 连续匹配加分
    let consecutive_bonus = count_consecutive_matches(query, target);
    score += consecutive_bonus * 10;

    // 匹配位置靠前加分
    let first_match_pos = find_first_match_position(query, target);
    score -= (first_match_pos * 5) as u32;

    // 长度比例
    let ratio = query.len() as f32 / target.len() as f32;
    score += (ratio * 20.0) as u32;

    score
}

/// 计算连续匹配数量
fn count_consecutive_matches(query: &str, target: &str) -> u32 {
    let query_chars: Vec<char> = query.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    let mut max_consecutive = 0u32;
    let mut current_consecutive = 0u32;
    let mut query_idx = 0usize;

    for target_char in &target_chars {
        if query_idx < query_chars.len() && *target_char == query_chars[query_idx] {
            current_consecutive += 1;
            query_idx += 1;
        } else {
            max_consecutive = max_consecutive.max(current_consecutive);
            current_consecutive = 0;
        }
    }

    max_consecutive.max(current_consecutive)
}

/// 找到第一个匹配字符的位置
fn find_first_match_position(query: &str, target: &str) -> usize {
    if let Some(first_char) = query.chars().next() {
        target.find(first_char).unwrap_or(target.len())
    } else {
        0
    }
}

/// 高亮匹配字符
///
/// 返回带有高亮标记的字符串
pub fn highlight_matches(query: &str, target: &str) -> String {
    if query.is_empty() {
        return target.to_string();
    }

    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    let mut result = String::new();
    let mut query_chars = query_lower.chars();
    let mut current_qc = query_chars.next();

    for (tc, original_tc) in target_lower.chars().zip(target.chars()) {
        if let Some(qc) = current_qc {
            if tc == qc {
                result.push('[');
                result.push(original_tc);
                result.push(']');
                current_qc = query_chars.next();
            } else {
                result.push(original_tc);
            }
        } else {
            result.push(original_tc);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        let (matched, score) = fuzzy_match("chrome", "Google Chrome");
        assert!(matched);
        assert!(score > 100);
    }

    #[test]
    fn test_fuzzy_match_partial() {
        let (matched, score) = fuzzy_match("gc", "Google Chrome");
        assert!(matched);
        assert!(score > 0);
    }

    #[test]
    fn test_fuzzy_match_fail() {
        let (matched, _) = fuzzy_match("xyz", "Google Chrome");
        assert!(!matched);
    }

    #[test]
    fn test_highlight() {
        let highlighted = highlight_matches("gc", "Google Chrome");
        assert!(highlighted.contains('['));
        assert!(highlighted.contains(']'));
    }
}
