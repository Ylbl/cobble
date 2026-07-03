import type { Session } from "../types/gallery";

export const mockSessions: Session[] = [
  {
    id: "aspirin-structure",
    title: "阿司匹林化学结构讨论",
    source: "ZCode",
    projectName: "ZCodeProject",
    projectPath: "C:\\Users\\o\\Desktop\\ZCodeProject",
    updatedAt: "1 分钟前",
    artifactCount: 4,
    turns: [
      {
        id: "aspirin-turn-1",
        index: 1,
        hint: "生成阿司匹林结构式",
        collapsed: true,
        artifacts: [
          {
            id: "aspirin-pdf",
            title: "阿司匹林结构式.pdf",
            kind: "pdf",
            status: "finished",
            previewType: "large",
          },
          {
            id: "aspirin-png",
            title: "阿司匹林合成反应.png",
            kind: "png",
            status: "finished",
            previewType: "small",
          },
        ],
      },
      {
        id: "aspirin-turn-2",
        index: 2,
        hint: "补充反应式和官能团说明",
        collapsed: true,
        artifacts: [
          {
            id: "aspirin-svg",
            title: "苯环取代反应.svg",
            kind: "svg",
            status: "compiling",
            previewType: "large",
          },
        ],
      },
      {
        id: "aspirin-turn-3",
        index: 3,
        hint: "复杂电路图",
        collapsed: false,
        artifacts: [
          {
            id: "rc-circuit",
            title: "RC 电路图.pdf",
            kind: "pdf",
            status: "failed",
            previewType: "small",
          },
        ],
      },
    ],
  },
  {
    id: "rlc-lab",
    title: "RLC 实验报告图像整理",
    source: "Codex",
    projectName: "PhysicsLab",
    projectPath: "C:\\Users\\o\\Documents\\PhysicsLab",
    updatedAt: "18 分钟前",
    artifactCount: 6,
    turns: [
      {
        id: "rlc-turn-1",
        index: 1,
        hint: "整理示波器截图和报告 PDF",
        collapsed: true,
        artifacts: [
          {
            id: "rlc-report",
            title: "RLC_实验报告.pdf",
            kind: "pdf",
            status: "finished",
            previewType: "large",
          },
          {
            id: "rlc-scope",
            title: "示波器波形.png",
            kind: "png",
            status: "finished",
            previewType: "small",
          },
        ],
      },
      {
        id: "rlc-turn-2",
        index: 2,
        hint: "生成相位关系示意",
        collapsed: false,
        artifacts: [
          {
            id: "rlc-phase",
            title: "相位关系.svg",
            kind: "svg",
            status: "finished",
            previewType: "large",
          },
        ],
      },
    ],
  },
  {
    id: "resume-assets",
    title: "简历附件 PDF 版本管理",
    source: "Cursor",
    projectName: "CareerDocs",
    projectPath: "C:\\Users\\o\\Desktop\\CareerDocs",
    updatedAt: "1 小时前",
    artifactCount: 3,
    turns: [
      {
        id: "resume-turn-1",
        index: 1,
        hint: "导出中文与英文简历",
        collapsed: false,
        artifacts: [
          {
            id: "resume-cn",
            title: "resume_zh.pdf",
            kind: "pdf",
            status: "finished",
            previewType: "large",
          },
          {
            id: "resume-en",
            title: "resume_en.pdf",
            kind: "pdf",
            status: "finished",
            previewType: "small",
          },
        ],
      },
    ],
  },
  {
    id: "latex-paper",
    title: "LaTeX 论文图表预览",
    source: "ZCode",
    projectName: "PaperDraft",
    projectPath: "C:\\Users\\o\\Research\\PaperDraft",
    updatedAt: "昨天",
    artifactCount: 5,
    turns: [
      {
        id: "paper-turn-1",
        index: 1,
        hint: "编译主文档和公式图",
        collapsed: false,
        artifacts: [
          {
            id: "paper-main",
            title: "main.pdf",
            kind: "latex",
            status: "compiling",
            previewType: "large",
          },
        ],
      },
    ],
  },
  {
    id: "ui-board",
    title: "AI Sidecar UI 参考板",
    source: "Codex",
    projectName: "Cobble",
    projectPath: "C:\\Users\\o\\RustroverProjects\\cobble",
    updatedAt: "2 天前",
    artifactCount: 8,
    turns: [
      {
        id: "ui-turn-1",
        index: 1,
        hint: "收集深色界面截图",
        collapsed: false,
        artifacts: [
          {
            id: "ui-shot",
            title: "dark-sidecar-reference.png",
            kind: "png",
            status: "finished",
            previewType: "large",
          },
        ],
      },
    ],
  },
];
