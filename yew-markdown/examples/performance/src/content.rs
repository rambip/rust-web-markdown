pub static MATH1: &str = r#"
# Maths
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
"#;
pub static MATH2: &str = r#"
# Maths
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
$$\frac{\Gamma\left(\frac s 2\right)\zeta(s)}{\pi^{s/2}} = \sum_{n=1}^\infty \int_0^\infty x^{{s\over 2}-1} e^{-n^2 \pi x}\, dx = \int_0^\infty x^{{s\over 2}-1} \sum_{n=1}^\infty e^{-n^2 \pi x}\, dx$$
"#;

pub static CODE: &str = include_str!("./main.rs");

pub fn generate_content() -> Vec<String> {
    let mut content = Vec::new();
    content.push("ready ?".to_string());
    content.push(MATH1.to_string());
    content.push(MATH2.to_string());
    content.push(format!("```rust\n{}```", CODE));
    content.push(format!(
        "just added one line of text \n```rust\n{}```",
        CODE
    ));
    content
}
