// https://www.youtube.com/watch?v=LULQtc5CUJ8&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT&index=2


use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
//use std::borrow::Cow;

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::DX12);
    // usamos la palabra clave unsafe para crear una surface renderizada para una ventana winit:
    // Esta surface proporciona una funcionalidad de dibujo para la plataforma compatible con winit.
    let surface = unsafe { instance.create_surface(&window) };

    // Dado que la API de wgpu es asíncrona cuando interactúa con la GPU, debemos colocar nuestro código de
    // renderizado dentro de la función asíncrona run, que se ejecuta cuando se carga el código.
    // Dentro de esta función asíncrona run, podemos acceder a la GPU en wgpu llamando a la función request adapter.
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("No se pudo encontrar un adaptador apropiado");

    // Aquí se requiere que el campo compatible surface sea renderizable con el adaptador solicitado.
    // Esto no crea la surface, solo garantiza que el adaptador pueda presentar en dicha superficie.
    // El campo force_fallback_adapter indica que si es true, solo se puede devolver un adaptador alternativo. Esto es
    // generalmente una implementación de "software" en el sistema. Una vez que tenemos el adaptador GPU,
    // podemos llamar a la función adapter.request_device para crear un dispositivo GPU.

    // Aquí, el atributo features de DeviceDescriptor nos permite especificar funciones adicionales.
    // Para este ejemplo de triángulo simple, no usamos ninguna función adicional.
    // El atributo limits describe el límite de ciertos tipos de recursos que creamos. Aquí, usamos
    // el valor predeterminado, que es compatible con la mayoría de los dispositivos.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("No se pudo crear el dispositivo");

    // Tal como se mencionó en la sección anterior, usamos la palabra clave unsafe para crear una surface de renderizado,
    // que es la parte de la ventana en la que dibujaremos. Necesitamos esta surface para escribir la salida desde
    // el fragment shader directamente a la pantalla.
    // A continuación, usamos el adaptador wgpu para configurar la surface con el siguiente fragmento de código.

    // Aquí, primero introducimos el parámetro de formato que define cómo se almacenará la textura de la surface en la
    // GPU. Diferentes pantallas prefieren diferentes formatos, por lo que llamamos a la función surface.get_preferred
    // para averiguar el mejor formato para usar en función de la pantalla que estamos usando.
    // Dentro de la estructura SurfaceConfiguration, el atributo usage describe cómo serán usadas las texturas de la
    // surface. RENDER ATTACHMENT especifica que las texturas se utilizarán para escribir en la superficie definida
    // en la ventana.

    // Los atributos width y height definen el tamaño de la textura de la surface, que normalmente debería ser el ancho
    // y altura de la ventana winit. Asegúrese de que el ancho y la altura de la textura de la surface no sean cero.
    // De lo contrario, puede hacer que su aplicación se bloquee.
    // El atributo present_mode utiliza la enumeración wgpu::PresentMode::Mailbox, que determina cómo
    // sincronizar la surface con la pantalla. La enumeración PresentMode tiene tres opciones:

    // • Inmediato. El motor de presentación no espera un período de borrado vertical y la solicitud es
    // presentado inmediatamente. Este es un modo de presentación de baja latencia, pero se pueden observar desgarros visibles.
    // Recurrirá a Fifo si no está disponible en la plataforma y el backend seleccionados. No es óptimo para dispositivos móviles.

    // • Mailbox. El motor de presentación espera el siguiente período de borrado vertical para actualizar la imagen actual,
    // pero los frames pueden enviarse sin demora. Este es un modo de presentación de baja latencia y
    // no se observarán desgarros visibles. Recurrirá a Fifo si no está disponible en la plataforma seleccionada y
    // back-end No es óptimo para dispositivos móviles.

    // • Fifo'. El motor de presentación espera el siguiente período de borrado vertical para actualizar la imagen actual.
    // La velocidad de fotogramas se limitará a la frecuencia de actualización de la pantalla, correspondiente a VSync.
    // No se observa rasgado. Óptimo para móvil.

    // Como se mencionó, aquí usamos la enumeración Mailbox.
    let format = surface.get_supported_formats(&adapter)[0];
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &config);

    // A continuación, cargamos los shaders que implementaremos en el archivo first triangle.wgsl usando el
    // La función create shader_module puede crear un módulo shader desde código fuente SP1R-V o WGSL.
    // Aquí, utilizaremos el módulo de shader oficial de WGSL.
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // Lo que hemos hecho hasta ahora son pasos básicos de inicialización, incluida la configuración del adaptador GPU,
    // GPU device y textura de la surface. Todos estos objetos son comunes a cualquier aplicación wgpu y no se
    // necesita ser reiniciados o cambiados. Sin embargo, después de esta inicialización, la canalización de renderizado
    // y el programa shader diferirá dependiendo de la aplicación.

    // La canalización en la API de wgpu describe todas las acciones que realizará la GPU cuando actúe sobre un conjunto de
    // datos. Por lo general, consta de dos estados programables, el vertex shader el fragment shader, similar a
    // WebGL. La API wgpu también agrega soporte para compute shaders, que existen fuera de la canalización de renderizado.
    //
    // Para renderizar el triángulo, necesitamos configurar esta canalización, crear los shaders y especificar atributos de vértice.
    // En wgpu, el objeto render_pipeline se usa para especificar las diferentes partes de la canalización.
    // La configuración de los componentes de esta canalización, como los shaders, el estado del vértice y el estado de
    // la salida de renderizado, son fijos, lo que permite que la GPU optimice mejor el renderizado para la canalización.
    // Mientras tanto, el buffer o las texturas vinculadas se pueden cambiar a las entradas o salidas correspondientes.

    // Aquí, primero debemos crear una pipeline_layout, que es útil cuando necesitamos usar el búfer de la GPU. Para
    // nuestro ejemplo de triángulo simple, es solo un marcador de posición sin ningún contenido significativo:

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Ahora podemos crear la canalización de renderizado que combina los shaders, vertex, el fragment y la
    // configuración de salida llamando a la función device.create_render_pipeline

    // La canalización requiere los atributos de vertex y fragment, que corresponden al vertex shader y
    // al fragment shader respectivamente. Aquí, podemos especificar qué función dentro del shader debe llamarse,
    // que se conoce como el entry_point. Asignamos las funciones vs_main y fs_main, implementadas anteriormente
    // en el archivo first_triangle.wgsl, como atributos de entry_point para el vertex y fragment shader

    // El atributo buffers dentro de la sección vertex le dice a wgpu qué tipo de vértices queremos pasar al
    // vertex shader. Aquí, para nuestro ejemplo de triángulo simple, definimos los vértices en el vertex shader directamente,
    // así que simplemente dejamos este atributo vacío. Tendremos que colocar algo aquí cuando usemos el buffer de GPU
    // para almacenar datos de vértice.

    // El atributo fragment es técnicamente opcional, por lo que tenemos que envolverlo dentro de la variante de
    // enumeración Some(). Solo lo necesitamos si queremos almacenar datos de color en la textura de la surface.
    // El atributo targets dentro de la sección fragment le dice a wgpu qué formato de salida de color debe configurar.
    // Aquí, simplemente usamos el formato de surface, que especifica el conjunto de slots de salida y el formato
    // de textura. Nuestro fragmente shader tiene un solo slot de salida para el color, que escribiremos
    // directamente en la imagen de la textura de la superficie. Por lo tanto, especificamos un único
    // estado de color para una imagen con este formato.

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter, &shader, &pipeline_layout);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
            }
            Event::RedrawRequested(_) => {
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.062, b: 0.08, a: 1.0 }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_title("wgpu03: triangulo");
    env_logger::init();
    pollster::block_on(run(event_loop, window));
}