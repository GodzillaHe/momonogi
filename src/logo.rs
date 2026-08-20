use std::io::IsTerminal;

const PEACH: &[&str] = &[
    "             ///////",
    "          ///       // /////////",
    "        //////      //##########",
    "          ####//////##          ##",
    "         #    ######              #",
    "        #                          #",
    "        #                           #",
    "        #                           #",
    "        #                           #",
    "        #                          #",
    "         #                        #",
    "          ##                    ##",
    "            ###              ###",
    "               #####      ###",
    "                    ######",
];

const WORDMARK: &[&str] = &[
    "   __  ___                               _",
    "  /  |/  /__  __ _  ___  ___  ___  ___ _(_)",
    r" / /|_/ / _ \/  ' \/ _ \/ _ \/ _ \/ _` / /",
    r"/_/  /_/\___/_/_/_/\___/_//_/\___/\_, /_/",
    "                                 /___/",
];

const RESET: &str = "\x1b[0m";
const LEAF: &str = "\x1b[38;2;101;196;102m";
const FRUIT: &str = "\x1b[38;2;255;154;118m";
const PINK: &str = "\x1b[38;2;255;71;173m";

pub fn plain() -> String {
    PEACH
        .iter()
        .copied()
        .chain(std::iter::once(""))
        .chain(WORDMARK.iter().copied())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render() -> String {
    if !std::io::stdout().is_terminal()
        || std::env::var_os("NO_COLOR").is_some()
        || std::env::var("TERM").is_ok_and(|value| value == "dumb")
    {
        return plain();
    }

    let mut lines = Vec::new();
    for line in PEACH {
        let mut rendered = String::new();
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '/' || ch == '#' {
                rendered.push_str(if ch == '/' { LEAF } else { FRUIT });
                rendered.push(ch);
                while chars.peek() == Some(&ch) {
                    rendered.push(chars.next().expect("peeked character"));
                }
                rendered.push_str(RESET);
            } else {
                rendered.push(ch);
            }
        }
        lines.push(rendered);
    }
    lines.push(String::new());
    lines.extend(WORDMARK.iter().map(|line| format!("{PINK}{line}{RESET}")));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    #[test]
    fn plain_logo_is_ascii_and_contains_wordmark() {
        let logo = super::plain();
        assert!(logo.is_ascii());
        assert!(logo.contains("///////"));
        assert!(logo.contains("__  ___"));
    }
}
