use rllm::chat::MessageType;
use rllm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ChatRole},
};

pub async fn generate_msg(git_diff_content: &str) {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI) // or LLMBackend::Anthropic, LLMBackend::Ollama, LLMBackend::DeepSeek, LLMBackend::XAI, LLMBackend::Phind ...
        .api_key(std::env::var("OPENAI_API_KEY").unwrap_or("sk-TESTKEY".into()))
        .model("gpt-4o-mini") // or model("claude-3-5-sonnet-20240620") or model("grok-2-latest") or model("deepseek-chat") or model("llama3.1") or model("Phind-70B") ...
        .max_tokens(1000)
        .temperature(0.7)
        .stream(false)
        .build()
        .expect("Failed to build LLM");
    let prompt = format!(
        r#"你是一个专业的版本控制助手，需要根据提供的 git diff 内容生成符合 Conventional Commits 规范的提交信息。请按以下步骤处理：

1. 分析代码变更内容：
   - 识别新增/删除的代码片段
   - 判断变更类型（feat/fix/chore/docs/style/refactor/test等）
   - 确定影响范围（模块/组件/功能）

2. 生成结构化信息：
   - 类型(type): 用英文小写开头
   - 范围(scope): 括号内的模块名称（可选）
   - 主题(subject): 50字内的简明描述
   - 正文(body): 说明变动背景和原因（可选）
   - 页脚(footer): 关联issue或PR（可选）

3. 遵守格式要求：
   - 首行不超过72字符
   - 使用命令式现在时态（"add" 而非 "added"）
   - 正文每行72字符换行
   - 空行分隔标题、正文和页脚

示例：
diff --git a/src/utils/date.js b/src/utils/date.js
+ export function formatTimestamp(ts) {{
+   return new Date(ts).toISOString().split('T')[0];
+ }}

生成：
feat(utils): add timestamp formatting function

Add standardized date formatting utility for consistent date display across UI components. Returns dates in YYYY-MM-DD format.

Closes #123

现在请处理以下 git diff 内容：
{}"#,
        git_diff_content
    );
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        message_type: MessageType::default(),
        content: prompt,
    }];

    let chat_resp = llm.chat(&messages).await;
    match chat_resp {
        Ok(text) => println!("Chat response:\n{}", text),
        Err(e) => eprintln!("Chat error: {}", e),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_generate_msg() {
        let diff_content = r#"diff --git a/calliper/devtools/task_debug.py b/calliper/devtools/task_debug.py
index 227a5684..62c19982 100644
--- a/calliper/devtools/task_debug.py
+++ b/calliper/devtools/task_debug.py
@@ -203,7 +203,7 @@ def import_env(
     app_secret="hello_calliper",
     exclude_domain=None,
     uid=-1,
-    tar_url="http://100.64.0.15:1208/api/v1/debug/cmp/{}/tar",
+    tar_url="http://100.64.0.12:1207/api/v1/debug/cmp/{}/tar",
 ):
     from calliper.security.authtoken import encode_url

diff --git a/calliper/handlers/handlers.py b/calliper/handlers/handlers.py
index 8090158d..d2b6ebb7 100644
--- a/calliper/handlers/handlers.py
+++ b/calliper/handlers/handlers.py
@@ -1386,7 +1386,8 @@ class ElementsCompareHandler(BaseHandler):
         data2 = NewStorage.interdoc_data(cmp_id, checksum2)
         pred1 = cls.delete_others(index1, data1)
         pred2 = cls.delete_others(index2, data2)
-        return diff_data(
+        return (
+            diff_data(
             pred1,
             pred2,
             {
@@ -1404,7 +1405,7 @@ class ElementsCompareHandler(BaseHandler):
                 "cmp_id": cmp_id,
                 "section_compare": True,
             },
-        )
+        ))

     @classmethod
     def delete_others(cls, ele_id, data):
diff --git a/misc/prod-requirements.txt b/misc/prod-requirements.txt
index 674cb4fb..90c8b65a 100644
--- a/misc/prod-requirements.txt
+++ b/misc/prod-requirements.txt
@@ -14,7 +14,7 @@ wtforms-tornado==0.0.2  # TODO too old, remove in future
 # ---- Calliper diff ----
 python-Levenshtein==0.23.0  # import Levenshtein
 attrs==23.2.0  # there's a bug after 24.1.0  https://github.com/python-attrs/attrs/issues/1348
-OpenCC==1.1.7  # 1.1.4 support py3.10; 1.1.5 support py3.11; 1.1.7 support py3.12
+#OpenCC==1.1.7  # 1.1.4 support py3.10; 1.1.5 support py3.11; 1.1.7 support py3.12
 # gensim==4.3.2  # not imported
 # jieba==0.42.1 # not imported

diff --git a/pkg/calliper_diff/dtw.py b/pkg/calliper_diff/dtw.py
index bba00147..9d9de893 100644
--- a/pkg/calliper_diff/dtw.py
+++ b/pkg/calliper_diff/dtw.py
@@ -4,9 +4,11 @@ from typing import List, Tuple
 import Levenshtein
 from numpy import array, ndarray, zeros

+from pkg.calliper_diff.our_counter import OurCounterReal
+

 # pylint: disable=too-many-locals
-def dtw(x, y, cal_sim, left2right=None, pages1=None, pages2=None, page_limit=5, max_num=10):
+def dtw(x: List[OurCounterReal], y: List[OurCounterReal], cal_sim, left2right=None, pages1=None, pages2=None, page_limit=5, max_num=10):
     r, c = len(x), len(y)
     D = zeros((r + 1, c + 1))
     for i in range(r):
"#;
        let result = generate_msg(diff_content);
    }
}