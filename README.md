
> [!WARNING]
> **ToldoUI-Engine** se encuentra actualmente en su fase **Beta 0.0.1 (totalmente inicial)**. 
> El proyecto puede sufrir cambios drásticos, contener errores o experimentar inestabilidad. Se recomienda usarlo con precaución y a manera de aprendisaje.


## 🛠️ Instalación y Requisitos

### Prerrequisitos

Para compilar y ejecutar ToldoUI-Engine necesitas tener instalado el compilador de Rust. Si no lo tienes, puedes instalarlo usando [rustup](https://rustup.rs/):

```bash
# En sistemas basados en Unix (macOS y Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

En **Windows**, descarga y ejecuta el instalador oficial desde [rustup.rs](https://rustup.rs/).

#### Dependencias del Sistema (Solo Linux)
Si estás desarrollando en Linux, asegúrate de tener instaladas las librerías necesarias de desarrollo para Winit y Font-Kit. Por ejemplo, en Ubuntu/Debian:

```bash
sudo apt-get update
sudo apt-get install -y libx11-dev libxft-dev libxmu-dev libdbus-1-dev libfontconfig1-dev
```

### Clonar e Instalar

Clona el repositorio en tu máquina local:

```bash
git clone https://github.com/jeremyCY94/ToldoUI-Engine.git
cd ToldoUI-Engine
```

Compila el proyecto para comprobar que todo está configurado correctamente:

```bash
cargo build
```

---

## 🚀 Ejecución

Para iniciar el navegador de demostración que carga el HTML y CSS ubicados en el directorio `examples/`, simplemente ejecuta:

```bash
cargo run
```


ToldoUI-Engine
A lightweight, high-performance HTML/CSS rendering engine written in Rust for native desktop applications.

https://github.com/user-attachments/assets/4ca9c3d1-b8da-4f31-a0f8-d753cfc2feb9

ToldoUI-Engine es un motor de interfaz gráfica (GUI) de próxima generación desarrollado en Rust. Está diseñado como una alternativa ultra-ligera, segura y eficiente a frameworks pesados como Electron, permitiendo a los desarrolladores construir aplicaciones de escritorio nativas utilizando tecnologías web estándar (HTML5 y CSS3) con ejecutables pesando menos de 10Mb.

<img width="803" height="217" alt="imagen" src="https://github.com/user-attachments/assets/65394371-d918-49d4-8c85-46202d729175" />


Inspirado en la arquitectura modular de Servo y el manejo de ventanas nativo de Winit, ToldoUI-Engine actúa como un "toldo" estructural: una capa ligera y resistente que procesa el Layout web y lo renderiza con el máximo rendimiento del hardware, sin la sobrecarga de memoria de un navegador Chromium completo.
✨ Características Principales (Features)

    Rendimiento Nativo con Rust: Olvídate del consumo masivo de RAM. Al estar construido sobre Rust, garantiza seguridad de memoria en tiempo de compilación y una huella de recursos mínima.

    Layout Web Tradicional: Parser y motor de renderizado propio diseñado para interpretar HTML y CSS, permitiendo layouts fluidos, selectores y estilos modernos de forma nativa.

    Ventanas Multiplataforma: Integración directa con winit para el manejo eficiente de eventos, teclado, mouse y ventanas nativas en Windows, macOS y Linux.

    Adiós a Electron: Ejecuta interfaces web en el escritorio sin necesidad de embeber un navegador completo ni un entorno Node.js pesado. Todo compila en un único binario nativo.

    Arquitectura Modular: Inspirado en los conceptos de aislamiento y paralelismo de Servo, diseñado para procesar el árbol de renderizado (Render Tree) de forma eficiente.

🏗️ Arquitectura de Inspiración

El motor se apoya en los hombros de gigantes del ecosistema de Rust:

    Servo (Conceptos de Renderizado): Filosofía de procesamiento paralelo y eficiente de HTML/CSS.

    Winit (Ventanas y Eventos): Manejo de la capa del sistema operativo y bucle de eventos (Event Loop) nativo.

🎯 ¿Para quién es este proyecto?

Para desarrolladores que aman la flexibilidad y velocidad de diseño que ofrecen HTML y CSS, pero que se niegan a sacrificar el rendimiento, la velocidad de arranque y el consumo de memoria de sus aplicaciones de escritorio.
