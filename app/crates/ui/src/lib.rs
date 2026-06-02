use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use constants::{LOCALES, TIMEZONES, KEYMAPS, DMS, GPUS};
use state::{AppField, AppState, Step};

pub fn draw_ui(f: &mut Frame, s: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.area());

    let step_text = match s.step {
        Step::Wifi => "0. Conexão Wi-Fi",
        Step::Configure => "1. Configurar Base",
        Step::Review => "2. Revisar Configurações",
        Step::Install => "3. Instalando Sistema",
        Step::Success => "4. Concluído",
    };

    let header = Paragraph::new(format!("⬡ Archism Launcher  |  Etapa Atual: {}", step_text))
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
    f.render_widget(header, chunks[0]);

    match s.step {
        Step::Wifi => {
            let wifi_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            let items: Vec<ListItem> = s.wifi_list.iter().enumerate().map(|(idx, net)| {
                let style = if idx == s.wifi_idx {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("  • {}", net)).style(style)
            }).collect();

            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Selecione sua rede sem fio (Wi-Fi)"));
            f.render_widget(list, wifi_chunks[0]);

            let pass_style = if s.wifi_input_mode {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let stars = "*".repeat(s.wifi_password.len());
            let pass_box = Paragraph::new(stars)
                .block(Block::default().borders(Borders::ALL).title("Senha da Rede Selecionada (Pressione ENTER para digitar)").style(pass_style));
            f.render_widget(pass_box, wifi_chunks[1]);

            let status_box = Paragraph::new(s.wifi_status.as_str())
                .block(Block::default().borders(Borders::ALL).title("Status da Conexão"));
            f.render_widget(status_box, wifi_chunks[2]);
        }
        Step::Configure => {
            let form_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), Constraint::Length(3), Constraint::Length(3),
                    Constraint::Length(3), Constraint::Length(3), Constraint::Length(3),
                    Constraint::Length(3), Constraint::Length(3), Constraint::Length(3),
                ].as_ref())
                .split(chunks[1]);

            let fields = [
                (AppField::Disk, format!("Disco de Destino: {}", s.disks.get(s.disk_idx).unwrap_or(&"Nenhum".to_string()))),
                (AppField::Hostname, format!("Nome da Máquina (Hostname): {}", s.hostname)),
                (AppField::Username, format!("Nome de Usuário (Username): {}", s.username)),
                (AppField::Locale, format!("Localização (Locale): {}", LOCALES[s.locale_idx])),
                (AppField::Timezone, format!("Fuso Horário (Timezone): {}", TIMEZONES[s.timezone_idx])),
                (AppField::Keymap, format!("Layout do Teclado (Keymap): {}", KEYMAPS[s.keymap_idx])),
                (AppField::SwapSize, format!("Memória SWAP: {} GB", s.swap_size)),
                (AppField::Dm, format!("Ambiente Gráfico (Interface): {}", DMS[s.dm_idx].to_uppercase())),
                (AppField::Gpu, format!("Driver de Vídeo (GPU): {}", GPUS[s.gpu_idx].to_uppercase())),
            ];

            for (field, text) in fields.iter() {
                let idx = match field {
                    AppField::Disk => 0, AppField::Hostname => 1, AppField::Username => 2,
                    AppField::Locale => 3, AppField::Timezone => 4, AppField::Keymap => 5,
                    AppField::SwapSize => 6, AppField::Dm => 7, AppField::Gpu => 8,
                    _ => 0,
                };
                let is_active = s.active_field == *field;
                let style = if is_active {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let p = Paragraph::new(text.as_str()).block(Block::default().borders(Borders::ALL).style(style));
                f.render_widget(p, form_chunks[idx]);
            }
        }
        Step::Review => {
            let review_text = format!(
                "Por favor confirme os dados estruturais antes da formatação:\n\n\
                 • Disco Alvo: {}\n\
                 • Hostname  : {}\n\
                 • Usuário   : {}\n\
                 • Locale    : {}\n\
                 • Timezone  : {}\n\
                 • Keymap    : {}\n\
                 • Swap File : {} GB\n\
                 • Interface : {}\n\
                 • Driver GPU: {}\n\n\
                 ATENÇÃO: Prosseguir apagará todas as partições existentes no disco selecionado.",
                s.disks.get(s.disk_idx).unwrap_or(&"".to_string()),
                s.hostname, s.username, LOCALES[s.locale_idx], TIMEZONES[s.timezone_idx],
                KEYMAPS[s.keymap_idx], s.swap_size, DMS[s.dm_idx].to_uppercase(), GPUS[s.gpu_idx].to_uppercase()
            );
            let p = Paragraph::new(review_text).block(Block::default().borders(Borders::ALL).title("Revisão"));
            f.render_widget(p, chunks[1]);
        }
        Step::Install => {
            let install_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(chunks[1]);

            let label = format!("{}% - {}", s.progress_percent, s.progress_stage);
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Progresso Geral"))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(s.progress_percent as u16)
                .label(label);
            f.render_widget(gauge, install_chunks[0]);

            let log_items: Vec<ListItem> = s.logs.iter().rev().take(40).map(|l| {
                let color = if l.starts_with('✓') {
                    Color::Green
                } else if l.starts_with("[ERROR]") {
                    Color::Red
                } else {
                    Color::Gray
                };
                ListItem::new(l.as_str()).style(Style::default().fg(color))
            }).collect();

            let list = List::new(log_items).block(Block::default().borders(Borders::ALL).title("Terminal Output Pipelines"));
            f.render_widget(list, install_chunks[1]);
        }
        Step::Success => {
            let txt = "✓ Instalação concluída com sucesso!\n\nO Arch Linux foi gravado no dispositivo.\n\nPressione [ENTER] para reiniciar a máquina.";
            let p = Paragraph::new(txt).block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Green)));
            f.render_widget(p, chunks[1]);
        }
    }

    let footer_text = match s.step {
        Step::Wifi => " [Seta Cima/Baixo] Escolher Rede | [Enter] Confirmar ou Abrir Senha | [S] Pular conexão ",
        Step::Configure => " [Seta Cima/Baixo] Navegar | [Seta Esq/Dir] Alterar Opções | [Enter] Avançar ",
        Step::Review => " [ESC] Voltar | [Enter] APAGAR DISCO E INSTALAR ARCH ",
        Step::Install => " Instalando pacotes e compilando chroot scripts... Por favor aguarde. ",
        Step::Success => " [Enter] Reiniciar o Sistema ",
    };
    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}