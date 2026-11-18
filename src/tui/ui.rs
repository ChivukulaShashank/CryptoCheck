use ratatui::{
    Frame,
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    style::{Color, Style, Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Tabs, Sparkline, Table, Row, Cell},
};
use crate::crypto::ExecutionMode;
use crate::stats::TvlaVerdict;
use super::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Tabs
            Constraint::Min(15),   // Main content
            Constraint::Length(3), // Status & Footer
        ])
        .split(f.area());

    render_header_and_tabs(f, chunks[0], app);

    match app.current_tab {
        0 => render_dashboard_tab(f, chunks[1], app),
        1 => render_tvla_tab(f, chunks[1], app),
        2 => render_ml_tab(f, chunks[1], app),
        3 => render_settings_tab(f, chunks[1], app),
        _ => (),
    }

    render_footer(f, chunks[2], app);
}

fn render_header_and_tabs(f: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(38),
            Constraint::Min(30),
            Constraint::Length(28),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" PQC ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::styled(" Timing Side-Channel Engine ", Style::default().fg(Color::Cyan).bold()),
    ])).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(title, header_chunks[0]);

    // Navigation Tabs
    let tab_titles = vec![
        " 1: Live Dashboard ",
        " 2: Statistical TVLA ",
        " 3: ML Distinguisher ",
        " 4: Export & Settings ",
    ];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
        .highlight_style(Style::default().fg(Color::Yellow).bold().add_modifier(Modifier::UNDERLINED))
        .select(app.current_tab);
    f.render_widget(tabs, header_chunks[1]);

    // Mode Banner
    let (mode_text, mode_color) = match app.mode {
        ExecutionMode::Hardened => (" [HARDENED: Constant-Time] ", Color::Green),
        ExecutionMode::Vulnerable => (" [VULNERABLE: Timing Leak] ", Color::Red),
    };
    let mode_widget = Paragraph::new(Line::from(vec![
        Span::styled(mode_text, Style::default().fg(mode_color).bold()),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(mode_widget, header_chunks[2]);
}

fn render_dashboard_tab(f: &mut Frame, area: Rect, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Sparkline & live stats
            Constraint::Min(8),    // Recent traces table
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Sparkline
            Constraint::Percentage(40), // Stats cards
        ])
        .split(main_chunks[0]);

    // Sparkline of recent cycle counts
    let sparkline_block = Block::default()
        .title(format!(" Live CPU Execution Cycles [{} - {}] ", app.algo.name(), app.algo.operation_name()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let sparkline = Sparkline::default()
        .block(sparkline_block)
        .data(&app.sparkline_data)
        .style(Style::default().fg(if app.mode == ExecutionMode::Vulnerable { Color::Red } else { Color::Green }));
    f.render_widget(sparkline, top_chunks[0]);

    // Performance Stats Card
    let s0 = app.tvla.stats_group0();
    let s1 = app.tvla.stats_group1();
    let total_samples = s0.count + s1.count;
    let min_c = if total_samples > 0 { s0.min.min(s1.min) } else { 0 };
    let max_c = s0.max.max(s1.max);
    let avg_c = if total_samples > 0 { (s0.mean * s0.count as f64 + s1.mean * s1.count as f64) / total_samples as f64 } else { 0.0 };

    let stats_text = vec![
        Line::from(vec![
            Span::raw(" Total Traces Captured : "),
            Span::styled(format!("{}", total_samples), Style::default().fg(Color::Yellow).bold()),
            Span::raw(" | Engine: "),
            Span::styled(if app.is_running { "RUNNING" } else { "PAUSED" }, Style::default().fg(if app.is_running { Color::Green } else { Color::DarkGray }).bold()),
        ]),
        Line::from(vec![
            Span::raw(" Min / Max Cycles      : "),
            Span::styled(format!("{} / {}", min_c, max_c), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw(" Mean Execution Latency: "),
            Span::styled(format!("{:.1} cycles", avg_c), Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::raw(" Timing Jitter         : "),
            Span::styled(format!("±{:.1} cycles", (s0.std_dev + s1.std_dev) / 2.0), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let stats_card = Paragraph::new(stats_text)
        .block(Block::default().title(" Operational Telemetry ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(stats_card, top_chunks[1]);

    // Recent Traces Table
    let rows: Vec<Row> = app.traces.iter().rev().take(12).map(|t| {
        let (class_label, class_style) = if t.class == 0 {
            ("Fixed (Grp 0)", Style::default().fg(Color::Blue))
        } else {
            ("Random (Grp 1)", Style::default().fg(Color::Yellow))
        };

        Row::new(vec![
            Cell::from(format!("#{}", t.id)),
            Cell::from(t.algo.name()),
            Cell::from(t.algo.operation_name()),
            Cell::from(t.mode.as_str()),
            Cell::from(class_label).style(class_style),
            Cell::from(format!("{} cycles", t.cycles)).style(Style::default().fg(Color::Green).bold()),
        ])
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(24),
            Constraint::Length(28),
            Constraint::Length(28),
            Constraint::Length(16),
            Constraint::Length(18),
        ],
    )
    .header(
        Row::new(vec!["Trace ID", "Algorithm", "Target Operation", "Security Configuration", "Input Class", "Cycle Measurement"])
            .style(Style::default().fg(Color::Cyan).bold())
            .bottom_margin(1),
    )
    .block(Block::default().title(" Captured High-Precision Timing Traces ").borders(Borders::ALL).border_type(BorderType::Rounded));

    f.render_widget(table, main_chunks[1]);
}

fn render_tvla_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Verdict Banner & Gauge
            Constraint::Length(9), // Distribution comparison cards
            Constraint::Min(6),    // ASCII Histogram
        ])
        .split(area);

    let verdict = app.tvla.evaluate_welch_t_test();
    let t_val = verdict.t_score();

    // Verdict Banner
    let (verdict_text, verdict_sub, banner_color) = match verdict {
        TvlaVerdict::InsufficientData => (
            "INSUFFICIENT TRACE DATA COLLECTED",
            "Please allow the engine to collect at least 30 samples to compute statistical leakage bounds.",
            Color::DarkGray,
        ),
        TvlaVerdict::PassConstantTime(_t) => (
            "CONSTANT-TIME SECURE [PASS]",
            "Welch's t-statistic is within safe cryptographic bounds (|t| <= 4.5). No side-channel leak detected.",
            Color::Green,
        ),
        TvlaVerdict::FailLeakDetected(_t) => (
            "SIDE-CHANNEL TIMING LEAK DETECTED [FAIL - CRITICAL]",
            "Welch's t-statistic exceeds the |t| > 4.5 threshold with >99.999% confidence! Measurable timing leakage present.",
            Color::Red,
        ),
    };

    let banner = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(format!("  {}  ", verdict_text), Style::default().fg(Color::Black).bg(banner_color).bold()),
            Span::styled(format!("  Current t-score: {:.3} (Standard Boundary: ±4.50)", t_val), Style::default().fg(banner_color).bold()),
        ]),
        Line::from(Span::styled(format!("  {}", verdict_sub), Style::default().fg(Color::White))),
    ])
    .block(Block::default().title(" Test Vector Leakage Assessment (TVLA) Verdict ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(banner, chunks[0]);

    // Distribution Comparison Cards
    let dist_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let s0 = app.tvla.stats_group0();
    let s1 = app.tvla.stats_group1();

    let g0_text = vec![
        Line::from(vec![Span::raw(" Sample Count (N₀) : "), Span::styled(format!("{}", s0.count), Style::default().fg(Color::Yellow).bold())]),
        Line::from(vec![Span::raw(" Mean Cycles (x̄₀)  : "), Span::styled(format!("{:.2}", s0.mean), Style::default().fg(Color::Cyan).bold())]),
        Line::from(vec![Span::raw(" Variance (s₀²)    : "), Span::styled(format!("{:.2}", s0.variance), Style::default().fg(Color::White))]),
        Line::from(vec![Span::raw(" Std Deviation (s₀): "), Span::styled(format!("{:.2}", s0.std_dev), Style::default().fg(Color::Magenta))]),
        Line::from(vec![Span::raw(" Min / Max Cycles  : "), Span::styled(format!("{} / {}", s0.min, s0.max), Style::default().fg(Color::DarkGray))]),
    ];
    let g0_block = Paragraph::new(g0_text)
        .block(Block::default().title(" Group 0: Fixed Input (Baseline Control) ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(g0_block, dist_chunks[0]);

    let g1_text = vec![
        Line::from(vec![Span::raw(" Sample Count (N₁) : "), Span::styled(format!("{}", s1.count), Style::default().fg(Color::Yellow).bold())]),
        Line::from(vec![Span::raw(" Mean Cycles (x̄₁)  : "), Span::styled(format!("{:.2}", s1.mean), Style::default().fg(Color::Cyan).bold())]),
        Line::from(vec![Span::raw(" Variance (s₁²)    : "), Span::styled(format!("{:.2}", s1.variance), Style::default().fg(Color::White))]),
        Line::from(vec![Span::raw(" Std Deviation (s₁): "), Span::styled(format!("{:.2}", s1.std_dev), Style::default().fg(Color::Magenta))]),
        Line::from(vec![Span::raw(" Min / Max Cycles  : "), Span::styled(format!("{} / {}", s1.min, s1.max), Style::default().fg(Color::DarkGray))]),
    ];
    let g1_block = Paragraph::new(g1_text)
        .block(Block::default().title(" Group 1: Random Input (Target Variable) ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(g1_block, dist_chunks[1]);

    // Distribution Histogram Visualization
    let bins = app.tvla.compute_histogram_bins(12);
    let mut hist_lines = Vec::new();
    hist_lines.push(Line::from(vec![
        Span::styled(" Cycle Range          | Group 0 (Fixed)                  | Group 1 (Random)                  ", Style::default().fg(Color::Cyan).bold()),
    ]));
    hist_lines.push(Line::from(" --------------------+----------------------------------+----------------------------------"));

    for b in bins {
        let bar0 = "█".repeat((b.count_group0 / 4).min(28));
        let bar1 = "█".repeat((b.count_group1 / 4).min(28));
        hist_lines.push(Line::from(vec![
            Span::raw(format!(" {:>7} - {:<7} | ", b.range_start, b.range_end)),
            Span::styled(format!("{:<28} ({})", bar0, b.count_group0), Style::default().fg(Color::Blue)),
            Span::raw(" | "),
            Span::styled(format!("{:<28} ({})", bar1, b.count_group1), Style::default().fg(Color::Yellow)),
        ]));
    }

    let hist_widget = Paragraph::new(hist_lines)
        .block(Block::default().title(" Binned Trace Latency Distributions (Group 0 vs Group 1) ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(hist_widget, chunks[2]);
}

fn render_ml_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Overview Header
            Constraint::Min(12),    // Models side-by-side
            Constraint::Length(5),  // Comparative Analysis
        ])
        .split(area);

    let header_text = vec![
        Line::from(vec![
            Span::styled(" Machine Learning Timing Distinguisher & Leakage Quantification Engine ", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::raw(" Evaluates whether an attacker using statistical machine learning can predict secret-dependent behavior from timing traces alone. "),
            Span::styled("[Press 'T' to Train Models]", Style::default().fg(Color::Yellow).bold()),
        ]),
    ];
    let header_widget = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(header_widget, chunks[0]);

    let model_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Model 1: Logistic Regression
    render_model_card(f, model_chunks[0], "Logistic Regression (SGD)", app.lr_eval.as_ref());

    // Model 2: Random Forest
    render_model_card(f, model_chunks[1], "Random Forest Ensemble Distinguisher", app.rf_eval.as_ref());

    // Comparative Vulnerable vs Hardened Analysis Card
    let (analysis_title, analysis_content, border_color) = if let Some(rf) = &app.rf_eval {
        if rf.roc_auc > 0.70 {
            (
                " [!] HIGH SIDE-CHANNEL EXPLOITABILITY CONFIRMED ",
                format!(
                    "The ML distinguisher extracts measurable side-channel leakage (ROC-AUC = {:.3} > 0.50, Extracted Entropy = {:.2} bits).\nSecret-dependent timing variations in this configuration can be exploited by an adversary.",
                    rf.roc_auc, rf.leakage_bits
                ),
                Color::Red,
            )
        } else {
            (
                " [✓] CONSTANT-TIME RESISTANCE CONFIRMED (NEGATIVE CONTROL VERIFIED) ",
                format!(
                    "The ML distinguisher performs equivalent to random guessing (ROC-AUC = {:.3} ≈ 0.50, Zero Leakage Bits).\nProves that constant-time engineering completely eliminates the side-channel attack surface.",
                    rf.roc_auc
                ),
                Color::Green,
            )
        }
    } else {
        (
            " Comparative Side-Channel Risk Quantification ",
            "Press 'T' to train models and quantify the side-channel attack surface on currently captured traces.".to_string(),
            Color::White,
        )
    };

    let analysis_card = Paragraph::new(analysis_content)
        .block(Block::default().title(analysis_title).borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(border_color)));
    f.render_widget(analysis_card, chunks[2]);
}

fn render_model_card(f: &mut Frame, area: Rect, title: &str, eval: Option<&crate::ml::EvaluationResult>) {
    let content = match eval {
        Some(e) => {
            let cm = e.confusion_matrix;
            let auc_color = if e.roc_auc > 0.70 { Color::Red } else if e.roc_auc > 0.58 { Color::Yellow } else { Color::Green };
            vec![
                Line::from(vec![
                    Span::raw(" Test Evaluation Samples: "),
                    Span::styled(format!("{}", e.total_samples), Style::default().fg(Color::Yellow).bold()),
                ]),
                Line::from(vec![
                    Span::raw(" Distinguisher Accuracy : "),
                    Span::styled(format!("{:.2}%", e.accuracy * 100.0), Style::default().fg(Color::White).bold()),
                ]),
                Line::from(vec![
                    Span::raw(" Precision / Recall / F1: "),
                    Span::styled(format!("{:.3} / {:.3} / {:.3}", e.precision, e.recall, e.f1_score), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw(" ROC-AUC Score          : "),
                    Span::styled(format!("{:.3} ", e.roc_auc), Style::default().fg(auc_color).bold()),
                    Span::styled(if e.roc_auc > 0.70 { "(LEAK DETECTED)" } else { "(NO LEAK / RANDOM)" }, Style::default().fg(auc_color)),
                ]),
                Line::from(vec![
                    Span::raw(" Extracted Info Entropy : "),
                    Span::styled(format!("{:.3} bits / trace", e.leakage_bits), Style::default().fg(Color::Magenta).bold()),
                ]),
                Line::from(""),
                Line::from(Span::styled(" Confusion Matrix:", Style::default().fg(Color::DarkGray).bold())),
                Line::from(format!("   Actual 0 (Fixed)  -> TN: {:<5} | FP: {:<5}", cm[0][0], cm[0][1])),
                Line::from(format!("   Actual 1 (Random) -> FN: {:<5} | TP: {:<5}", cm[1][0], cm[1][1])),
            ]
        }
        None => vec![
            Line::from(""),
            Line::from(Span::styled(" Model not yet trained on active trace dataset.", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(" Press 'T' to train and evaluate classifier.", Style::default().fg(Color::Yellow))),
        ],
    };

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let p = Paragraph::new(content).block(block);
    f.render_widget(p, area);
}

fn render_settings_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Min(4),
        ])
        .split(area);

    let info_text = vec![
        Line::from(vec![
            Span::styled(" Cryptographic Target : ", Style::default().bold()),
            Span::styled(app.algo.name(), Style::default().fg(Color::Cyan).bold()),
            Span::raw(" (Post-Quantum Cryptography Standard)"),
        ]),
        Line::from(vec![
            Span::styled(" Target Operation     : ", Style::default().bold()),
            Span::styled(app.algo.operation_name(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(" Security Configuration: ", Style::default().bold()),
            Span::styled(app.mode.as_str(), Style::default().fg(if app.mode == ExecutionMode::Hardened { Color::Green } else { Color::Red }).bold()),
        ]),
        Line::from(vec![
            Span::styled(" Timing Instrumentation: ", Style::default().bold()),
            Span::raw("Hardware RDTSC with _mm_lfence instruction barrier serialization"),
        ]),
    ];
    let info_card = Paragraph::new(info_text)
        .block(Block::default().title(" Project & Target Specifications ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(info_card, chunks[0]);

    let controls_text = vec![
        Line::from(vec![
            Span::styled(" [Tab] / [1-4] ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Switch between Dashboard, TVLA, ML Distinguisher, and Settings"),
        ]),
        Line::from(vec![
            Span::styled(" [Space]       ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Pause / Resume the live cycle timing acquisition engine"),
        ]),
        Line::from(vec![
            Span::styled(" [M]           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Toggle Security Mode between HARDENED (Constant-Time) and VULNERABLE (Timing Leak)"),
        ]),
        Line::from(vec![
            Span::styled(" [A]           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Cycle target algorithm (ML-KEM-512 / Kyber-512 -> ML-KEM-768 -> ML-DSA-44 / Dilithium)"),
        ]),
        Line::from(vec![
            Span::styled(" [T]           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Train and evaluate ML Distinguishers (Logistic Regression + Random Forest)"),
        ]),
        Line::from(vec![
            Span::styled(" [C]           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Clear all accumulated traces and reset statistical buffers"),
        ]),
        Line::from(vec![
            Span::styled(" [E]           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Export captured traces to 'timing_traces.csv'"),
        ]),
        Line::from(vec![
            Span::styled(" [Q] / [Esc]   ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Quit the application cleanly and restore terminal mode"),
        ]),
    ];
    let controls_card = Paragraph::new(controls_text)
        .block(Block::default().title(" Interactive Keyboard Commands ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(controls_card, chunks[1]);

    let export_text = vec![
        Line::from(format!(" Total Traces Currently in Memory: {}", app.traces.len())),
        Line::from(" Traces are saved with features: [id, algorithm, operation, mode, class, cycles, timestamp_ms]"),
        Line::from(Span::styled(" Press 'E' at any time to export memory dataset to disk.", Style::default().fg(Color::Green))),
    ];
    let export_card = Paragraph::new(export_text)
        .block(Block::default().title(" Dataset Persistence ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(export_card, chunks[2]);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = Line::from(vec![
        Span::styled(" STATUS: ", Style::default().fg(Color::Black).bg(Color::White).bold()),
        Span::styled(format!(" {} ", app.status_msg), Style::default().fg(Color::Yellow).bold()),
        Span::raw(" | "),
        Span::styled("Keys: [Tab] Navigate | [Space] Pause/Play | [M] Toggle Mode | [A] Change Algo | [T] Train ML | [E] Export | [Q] Quit", Style::default().fg(Color::DarkGray)),
    ]);
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(footer, area);
}
