ToldoUI-Engine

    A lightweight, high-performance HTML/CSS rendering engine written in Rust for native desktop applications.
<img width="1038" height="823" alt="imagen" src="https://github.com/user-attachments/assets/9c29e01f-3dfd-47d7-b81d-723da25f477f" />

ToldoUI-Engine es un motor de interfaz gráfica (GUI) de próxima generación desarrollado en Rust. Está diseñado como una alternativa ultra-ligera, segura y eficiente a frameworks pesados como Electron, permitiendo a los desarrolladores construir aplicaciones de escritorio nativas utilizando tecnologías web estándar (HTML5 y CSS3).

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
