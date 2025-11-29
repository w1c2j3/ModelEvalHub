# Text2Image Benchmark Integration (placeholder)

This module will eventually integrate with [text2image-benchmark](https://github.com/boomb0om/text2image-benchmark) for T2I metrics such as FID and CLIPScore.

Planned steps:
1. Implement the shared `EvalRunner` trait for text-to-image workflows.
2. Support object-store based sample outputs (images) with metadata stored in DB.
3. Expose configuration knobs via `EvalConfig.task` and `EvalConfig.output`.

