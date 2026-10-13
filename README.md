# OA2IS V4 — Aceite y cantidad de sensores

Nueva solución independiente. Las versiones V1, V2, V3, experiments y outputs se conservaron mediante un inventario SHA-256 previo y comprobación posterior (preservation_report.json).

## Abrir en este PC

Ejecutar **INICIAR_V4.ps1** y abrir <http://127.0.0.1:8771/?oil=95&sensors=6>. El servidor queda en segundo plano. Si ya está funcionando, el iniciador comprueba que corresponde a V4. Presionar **Reproducir** para ver el mecanismo y el aceite; el dashboard está junto a la vista 3D. Desplazarse hacia abajo para ver todas las lecturas y el mapa temporal.

## Qué se puede variar

- Litros de aceite: 30–145 L, pasos de 5 L en la interfaz.
- Cantidad de sondas: 2–12, ubicaciones anidadas y fijas.
- Los controles de cámara, reproducción y aspecto cambian la presentación.

El motor siempre funciona a 300 rpm, con temperatura 60 °C, gravedad 9,81 m/s², sin inclinación ni aceleración adicional, densidad 900 kg/m³ y viscosidad 264,238 cSt. Cada caso calcula **2,4 segundos físicos**, 121 cuadros a 50 Hz. La API rechaza variables adicionales.

## Casos entregados y calculados

| Volumen | Partículas | Masa | Duración | Cantidad de sondas disponibles |
|---|---:|---:|---:|---|
| 55 L | 2098 | 49,5 kg | 2,400008 s | 2, 4, 6, 8, 12 |
| 95 L | 3624 | 85,5 kg | 2,400008 s | 2, 4, 6, 8, 12 |
| 125 L | 4768 | 112,5 kg | 2,400008 s | 2, 4, 6, 8, 12 |

Son **15 combinaciones experimentales y tres cálculos físicos independientes**. Las sondas son virtuales y no intrusivas: agregar canales reutiliza la trayectoria del mismo volumen. La masa, geometría, material y parámetros numéricos se conservan. Los 15 CSV están en data/case_exports.

Elegir un volumen nuevo y presionar Ver / calcular caso ejecuta el mismo motor físico. El cálculo tarda minutos; la reproducción posterior es interactiva. Las tres ejecuciones iniciales tardaron aproximadamente 5,1–6,9 minutos cada una, realizadas simultáneamente en este PC; el tiempo en otro servidor dependerá de su CPU. El cálculo usa CPU y la vista 3D usa WebGL2.

## Señales

Presencia local de aceite, aire inferido, respuesta temporal con ruido, capacitancia ilustrativa en pF, estado seco/mixto/aceite y nivel local por columna. Los gráficos muestran toda la ventana de 2,4 s y el cursor marca el cuadro mostrado. Descargar señales CSV exporta todos los cuadros del caso seleccionado. Añadir sensores conserva exactamente las señales de los canales existentes.

## Código abierto y portabilidad

Componentes integrados: Salva Apache-2.0; código propio y Three.js MIT; Node.js MIT con avisos de terceros; dependencias Rust abiertas; Noto Sans OFL-1.1. El motor es WebAssembly sin importaciones externas. No requiere Anaconda, CUDA, Python, npm ni un servicio comercial para funcionar. Ver THIRD_PARTY_NOTICES.md y open_source_audit.json.

En GNU/Linux o Windows con **Node.js 22 o posterior** y un navegador con WebGL2:

```sh
node server.mjs
```

Abrir <http://127.0.0.1:8771>. Todos los recursos y datos son locales; los enlaces bibliográficos son opcionales para consultar fuentes. El paquete portable incluye motor compilado, código y fuentes de sus dependencias. Node y el navegador se instalan aparte.

### Servidor externo

Copiar y descomprimir el paquete en un servidor Linux con Node 22+. Iniciar node server.mjs en ese servidor. Conectar mediante un túnel SSH:

```sh
ssh -L 8771:127.0.0.1:8771 usuario@servidor
```

Abrir localhost:8771 en el navegador del PC. De este modo la CPU del servidor hace el cálculo y el PC dibuja los resultados. Recomendación inicial: 4 núcleos, 4 GB RAM y 1 GB de almacenamiento disponible. No se exige GPU para calcular. Esa cifra es una recomendación, no una prueba de rendimiento en nube. No había un servidor externo disponible: se comprobó el paquete descomprimido con un segundo proceso local independiente.

## Reconstruir el motor

Rust y el target wasm32-unknown-unknown son herramientas abiertas de construcción, no requisitos para ejecutar los datos entregados. Desde esta carpeta:

```sh
rustup target add wasm32-unknown-unknown
cargo build --offline --locked --release --target wasm32-unknown-unknown --manifest-path engine/Cargo.toml
```

Copiar engine/target/wasm32-unknown-unknown/release/oa2is_oil_engine.wasm a dashboard/oil_engine.wasm. El target de Rust debe estar instalado antes de trabajar sin conexión. Salva está incluido en build/salva y las dependencias se encuentran en engine/vendor; .cargo/config.toml usa esas fuentes. Rust usado en esta entrega: 1.99.0. Tras cambiar el motor, recalcular los tres volúmenes con node build_cases.mjs y verificar con node verify.mjs; el servidor rechaza datos cuyo SHA no coincide con el motor.

## Verificación y límites

- Ocho pruebas Rust: montaje planetario, Willis, relación, velocidades de frontera, geometría, viscosidad y efecto del volumen.
- Tres ejecuciones: valores finitos, duración, masa, conteo, contención, ausencia de centros dentro de sólidos, límite de corrección geométrica y velocidad acotada.
- Quince combinaciones: número de sondas, señales reproducibles desde posiciones físicas, canales comunes idénticos.
- Prueba causal de fronteras móviles frente a quietas (solo QA); prueba del formulario, API y CSV; revisión visual del dashboard.

Los controles numéricos no certifican precisión física. El mecanismo usa una topología GE documentada y reducción cercana a 32,62, pero dimensiones/dientes ilustrativos y envolvente fija; no es CAD certificado del GDY106. El aire se infiere, no se calcula como fase gaseosa. No se resuelven película fina, espuma, tensión superficial, calentamiento ni desgaste. Es una representación macroscópica para estudiar redistribución y observación con sondas. Ver **research/sources.md** para evidencia, hipótesis y fuentes.
