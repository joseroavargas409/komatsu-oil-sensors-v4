# V4: qué se realizó, cómo, herramientas y fuentes

Documentación añadida el **13 de octubre de 2026**. Describe la versión independiente preservada del 12 de octubre. Esta actualización agrega documentación a GitHub; mantiene el motor, las trayectorias y el ZIP original.

## 1. Objetivo y referencia

Se desarrolló un estudio controlado de redistribución de aceite y observación con sondas. Solo se varían **litros de aceite** y **cantidad de sensores**, con el mecanismo funcionando y las demás condiciones constantes.

La referencia es la rueda motorizada del **Komatsu 930E-4SE**: el fabricante publica motor GDY106 de inducción AC, reducción 32,62:1 y 95 L por caja reductora. Estos datos orientan la arquitectura y el volumen nominal del estudio; no proporcionan el CAD interno. [Komatsu, folleto, página 2](https://www.komatsu.com/content/dam/komatsu/websites/oceania/documents/brochures/dump-trucks/930e-4se.pdf).

## 2. Qué se realizó

- Motor de aceite en Rust/Salva compilado a WebAssembly autónomo.
- Dos etapas planetarias con engranajes, ejes y portaplanetas que actúan como fronteras móviles del fluido.
- Tres cálculos físicos de **55, 95 y 125 L**, cada uno de **2,4 s** y 121 cuadros a 50 Hz.
- Quince combinaciones predefinidas al seleccionar 2, 4, 6, 8 o 12 sondas sobre cada trayectoria. La interfaz permite enteros de 2 a 12.
- Dashboard 3D, curvas de presencia de aceite, indicador de capacitancia, tabla de lecturas, mapa temporal y CSV.
- Aspecto visual de superficie suave y opción de partículas físicas.
- Paquete portable comprobado mediante un segundo proceso local; fuentes y licencias incluidas.

## 3. Cómo se implementó

1. Se genera una distribución inicial de partículas dentro de una carcasa cilíndrica. Su volumen se ajusta para representar exactamente los litros solicitados.
2. Salva aplica DFSPH para presión/incomprensibilidad. El código propio incorpora viscosidad laminar en unidades SI y un tratamiento aproximado de velocidad de pared reflejada. [Salva](https://github.com/dimforge/salva), [DFSPH](https://animation.rwth-aachen.de/publication/054/).
3. En cada paso se actualizan posición y velocidad de las fronteras; los planetas incorporan órbita y giro propio. Así el mecanismo transmite movimiento al aceite.
4. Se corrigen geométricamente los centros que penetran el metal o salen de la envolvente. La magnitud de esa corrección se registra como control numérico.
5. Se guardan posiciones y métricas. El modelo de sensores calcula ocupación por kernel, filtro temporal, ruido independiente por canal e histéresis.
6. Node.js entrega datos y ejecuta nuevos casos; Three.js reproduce las posiciones ya calculadas. La suavización visual afecta la presentación, mientras las sondas usan posiciones físicas.

### Condiciones del estudio

| Variable | Valor de V4 | Procedencia |
|---|---|---|
| Volumen | 30–145 L configurables; 55/95/125 L calculados | Condición del experimento |
| Sondas | 2–12 en posiciones fijas y anidadas | Modelo virtual propio |
| Velocidad del eje | 300 rpm | Supuesto constante |
| Temperatura del aceite | 60 °C | Supuesto constante |
| Densidad | 900 kg/m³ | Supuesto |
| Viscosidad cinemática | 264,238 cSt | Valor fijo inspirado en la curva de referencia del lubricante |
| Gravedad | 9,81 m/s², eje Z negativo | Condición constante |
| Inclinación / aceleración adicional | Cero | Condición constante |
| Carcasa | Radio 0,5 m y longitud 0,9 m, fija | Geometría ilustrativa |
| Discretización | Radio de partícula 16 mm; separación 32 mm; paso 1 ms | Parámetros numéricos |

Las etapas utilizan dientes ilustrativos 18/27/72 y 23/52/127. La ecuación de Willis con coronas fijas produce una reducción total **32,60869565:1**. La forma de dientes y las holguras requieren CAD y validación del equipo concreto. [Implementación y fuentes mecánicas](../research/sources.md).

### Sensores y señales

La ocupación de aceite se estima alrededor de cada sonda; el aire es `1 - ocupación`. Se aplica respuesta de 80 ms, ruido y umbrales. La capacitancia es un **indicador ilustrativo**, `20*(1 + 1,2*señal)` pF. La columna de aceite integra el campo completo; no es una lectura comercial de nivel global. Agregar sondas reutiliza el mismo fluido y conserva las señales de los canales anteriores. [Modelo de sensores](../dashboard/sensor-model.js).

## 4. Herramientas utilizadas y estado

| Herramienta | Uso | Estado / licencia |
|---|---|---|
| Rust y Cargo | Implementación, compilación y pruebas | Utilizados; toolchain registrado 1.99.0; avisos MIT/Apache y de terceros |
| Salva 0.10.0 | Solver SPH | Integrado, revisión `51c153e05974836eea17db445e30f1ff64bb5b32`; Apache-2.0 |
| WebAssembly | Motor distribuido y ejecutado por Node | Módulo autónomo, sin importaciones externas |
| Node.js 22+ | Servicio local y cálculo CPU | Utilizado; MIT y avisos de terceros |
| Three.js r180 / WebGL2 | Vista 3D y superficie | Integrado; Three.js MIT |
| JavaScript, HTML y CSS | Sondas, gráficos, formularios y CSV | Código propio MIT |
| Noto Sans | Tipografía local | Integrada; OFL-1.1 |
| Git y GitHub CLI | Publicación independiente | Utilizados; repositorio privado y ZIP original con SHA-256 |

Los paquetes Rust, sus versiones, fuentes y licencias están en [open_source_audit.json](../open_source_audit.json), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md), [engine/vendor](../engine/vendor/) y [licenses](../licenses/). El cálculo usa CPU; WebGL2 dibuja la escena en el navegador. El sistema operativo y el navegador son requisitos del equipo.

### Mapa del código

| Parte | Archivos |
|---|---|
| Motor y geometría física | [engine/src](../engine/src/) |
| Fuentes del solver | [build/salva](../build/salva/) |
| Ejecución de un caso | [run_case.mjs](../run_case.mjs) |
| API | [server.mjs](../server.mjs) |
| Sondas | [dashboard/sensor-model.js](../dashboard/sensor-model.js) |
| Datos y CSV | [data](../data/) |

## 5. Qué se comprobó

- Ocho pruebas Rust de montaje, Willis, velocidades, superficies, viscosidad y conteo según volumen. [Registro preservado de las pruebas](evidencias/pruebas_rust_v4.json).
- Los tres cálculos y quince combinaciones pasaron los controles definidos. [validation_report.json](../validation_report.json).
- La API restringe las variables a litros y cantidad de sondas. [api_validation_report.json](../api_validation_report.json).
- El ensayo corto de 95 L con mecanismo móvil y detenido produce una diferencia RMS de posiciones de aproximadamente **21,8 mm** a 0,1 s. Confirma que el mecanismo altera el cálculo. [coupling_report.json](../coupling_report.json).
- Las huellas de las fuentes publicadas están en [GITHUB_SNAPSHOT.json](../GITHUB_SNAPSHOT.json).

Estas comprobaciones verifican propiedades del programa y su estabilidad dentro de los límites establecidos. No cuantifican el error frente a un reductor real.

## 6. Fuentes y su aporte

| Fuente | Uso en el proyecto |
|---|---|
| [Komatsu 930E-4SE](https://www.komatsu.com/content/dam/komatsu/websites/oceania/documents/brochures/dump-trucks/930e-4se.pdf) | Arquitectura, capacidad y reducción publicadas |
| [Patente GE US7455616B2](https://patents.google.com/patent/US7455616B2/en) | Referencia de topología de doble reducción; no acredita dientes o CAD de un GDY106 concreto |
| [Salva, fuente oficial](https://github.com/dimforge/salva) | Solver utilizado y licencia |
| [Bender y Koschier, DFSPH, 2015](https://animation.rwth-aachen.de/publication/054/) | Fundamento del solver de presión y divergencia |
| [Morris, Fox y Zhu, 1997](https://doi.org/10.1006/jcph.1997.5776) | Referencia para el tratamiento viscoso por partículas |
| [Bel-Ray, producto](https://www.belray.com/product/synthetic-gear-oil/) y [ficha de referencia](https://www.belray.com/wp-content/uploads/2019/02/Synthetic-Gear-Oil.pdf) | Base de la aproximación de viscosidad del aceite ilustrativo |
| [Investigación preservada](../research/sources.md) | Trazabilidad del mecanismo, sensores, materiales del usuario y render de superficie |

Los documentos de capacitación aportados sirvieron para identificar componentes. Su edición y correspondencia exacta no están autenticadas. Las fuentes sustentan decisiones de modelado; no convierten los supuestos en parámetros OEM medidos.

## 7. Límites y reproducción

La vista de motor AC aporta contexto geométrico; en V4 el giro se prescribe. No se integra el circuito eléctrico de V3 ni la retroacción del aceite sobre el motor. El aire es inferido. No se resuelven película lubricante microscópica, espuma, cavitación, calor del aceite, desgaste o contacto mecánico entre dientes. La ventana de 2,4 s es transitoria.

Con Node.js instalado, ejecutar `node server.mjs` y abrir **http://127.0.0.1:8771/**. Los resultados guardados funcionan sin recompilar. [README](../README.md) explica reconstrucción, nuevas simulaciones y acceso por túnel SSH. El paquete se probó localmente; la ejecución en servidor externo queda sin comprobar.
