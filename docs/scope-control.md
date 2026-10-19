# Control formal de alcance

## Contratos

El alcance se prepara como borrador y se publica como una revisión conservada.
Guardar un borrador no retira la oferta publicada. El cliente acepta la última
revisión publicada vigente con su identidad autenticada y hora del
servidor. La primera aceptación activa el control del proyecto; nunca se genera
una aceptación por migrar datos. Las tareas incluidas en esa revisión quedan
clasificadas como base. La revisión conserva también las condiciones económicas
y los hitos originales; editar sus valores operativos no altera lo aceptado. Una tarea posterior necesita una clasificación explícita:
trabajo incluido (con justificación) o extra vinculado a una solicitud de cambio.
Las subtareas heredan el origen de su padre. No se puede quitar un vínculo para
eludir la aprobación ni mover trabajo clasificado a otro proyecto.

Las solicitudes mantienen su estado operativo y un historial de revisiones
inmutable. Cualquier cambio de contenido, evaluación, importe, horas, días o
destino temporal incrementa la versión. Editar una oferta publicada la devuelve
a `submitted`; requiere publicar otra evaluación. Aprobar/rechazar y publicar
evaluación requieren la versión vista. Las escrituras comparan esa versión en
PostgreSQL; una versión obsoleta devuelve HTTP 409. Repetir la misma decisión
sobre la misma oferta devuelve el resultado sin repetir efectos.

La aprobación crea, en la misma transacción que la decisión, una factura nueva
en borrador por el suplemento y aplica el plazo. El extra solo queda habilitado
cuando esos efectos existen. No se modifican facturas anteriores ni se cobran
pagos externos. Un suplemento es un precio fijo para ese extra, independiente
de la modalidad del proyecto (fijo, horas, retainer). Sus horas son una estimación
de esfuerzo; su tiempo se registra como no facturable para evitar duplicación.
Costo cero no crea factura y sigue requiriendo consentimiento. Un proyecto no
facturable solo admite ofertas de costo cero. La moneda se conserva al publicar
la oferta; modificar la moneda operativa del proyecto no cambia acuerdos previos.

Un impacto positivo en días necesita un hito del mismo proyecto con fecha.
La oferta conserva esa fecha y su nueva fecha; los días son días naturales calculados en UTC.
Solo se modifica ese hito. Si otra aprobación o edición cambió la fecha, aprobar
la oferta antigua falla y requiere reevaluación. No se suman extensiones a todas
las tareas. Un cambio sin impacto temporal no necesita hito. El cierre
`implemented` requiere tareas vinculadas y todas terminadas.

Eventos y revisiones publicados conservan actor, procedencia y fecha. Comentarios
de negociación se agregan como eventos, no sustituyen revisiones. La identidad
del cliente procede de `client_auth`, no de una IP enviada en el cuerpo.
Los avisos del portal son registros persistentes de pendientes con enlaces a
sus revisiones; no dependen de correo ni de WebSocket. Aprobar/rechazar los
resuelve dentro de la misma transacción.

## Archivo y conservación

No se pueden borrar solicitudes con historial, alcances, vínculos de trabajo ni
facturas derivadas. Se cancela o cierra el registro operativo; el acuerdo se
conserva. Las claves de historia impiden cascadas destructivas al eliminar
proyectos/clientes/propietarios. Los adjuntos publicados se conservan por ID y
metadatos; PostgreSQL bloquea su eliminación antes de borrar el objeto físico.
Los backups deben incluir la base y `UPLOAD_DIR` o el bucket. Los permisos de
administración de la base permiten mantenimiento deliberado; las garantías no
pretenden impedir acciones de un superusuario.

## Migración y despliegue

`cargo run --locked --bin migrate` inicializa bases nuevas o aplica pasos pendientes
bajo bloqueo advisory y transacción, registrando checksums. El paso 1 apunta al
`database/schema.sql` original y a su compatibilidad UUIDv7; el paso 2 ejecuta
`database/migrations/002_scope_control.sql`, que reúne las antiguas ampliaciones
001/002 sin incorporarlas al esquema base.

Reaplicar `schema.sql` a una base existente produce errores por tablas duplicadas.
Para una base antigua sin historial, `migrate` verifica su estructura contra un
esquema temporal construido desde el SQL base. Compara columnas, restricciones,
índices, vistas, funciones y triggers; si coincide reconoce el paso 1 sin repetir
sus CREATE y aplica solamente la ampliación. Conserva usuarios, contraseñas,
finanzas y estados históricos. Si no coincide, revierte y pide revisar la estructura.
Necesita permiso CREATE SCHEMA para ese reconocimiento. Historias experimentales
con checksums distintos no se reetiquetan; desarrollo puede usar el reset autorizado.

El backend comprueba la versión requerida sin migrar por sorpresa. Después del
lanzamiento no modificar los SQL registrados: añadir versiones incrementales. Los
SQL quedan embebidos en los binarios de la distribución self-hosted.

La migración no instala cuentas ni datos ficticios. El comando `seed` exige modo
desarrollo y `horbyta_development`; `--reset` reconstruye solo su esquema public.
Para licencias self-hosted: ejecutar `./migrate`, crear el administrador con
`./create-user` y arrancar el servidor. Usar backup de base y objetos antes de
actualizaciones. Las reglas históricas y el indicador `legacy` se mantienen en el
modelo para importaciones explícitas, sin inventar aceptaciones.

Ejecutar compilación/pruebas del backend con un job y un hilo; en Linux usar
`systemd-run --user --scope -p CPUQuota=50% -p MemoryMax=1536M -p MemorySwapMax=0 nice -n 19 cargo check --locked`. Se mantiene Rust fijado, Quasar independiente y Nginx, sin Docker.

### Procedimiento operativo

1. Reservar una ventana de mantenimiento y detener escrituras del backend. Conservar
   una copia del binario/configuración anterior y de los archivos del frontend.
2. Hacer un backup con `pg_dump --format=custom` de la base y una copia consistente
   de `UPLOAD_DIR` o del bucket. Guardar ambos fuera de la VPS y con acceso restringido.
   Restaurar primero en una base aparte con `pg_restore --exit-on-error` y comprobar
   proyectos, facturas, pagos y adjuntos; no restaurar sobre la base activa como ensayo.
3. Con la conexión de destino configurada, ejecutar
   `cargo run --locked --bin migrate`.
   El comando es transaccional e idempotente; un error revierte esa migración.
   Una migración ya desplegada se conserva intacta: los cambios posteriores van en
   otro archivo/versionado, nunca modificando su checksum registrado.
4. Publicar juntos el backend y el build Quasar compatible servido por Nginx.
   Los endpoints de decisiones ahora exigen `version`; un frontend anterior no
   tiene ese contrato. El backend verifica la migración antes de aceptar tráfico.
5. Comprobar el portal con una cuenta de prueba y vigilar los errores del worker
   de entrega. Las decisiones están confirmadas antes de notificar; el outbox
   reintenta después de 30 segundos y entrega como máximo diez eventos por ciclo.
   Si hay un problema de aplicación, detener las escrituras y corregirlo conservando
   acuerdos y efectos; volver a código anterior no elimina las tablas nuevas.

La implementación inicial A1–A7 no migró las tablas habituales. La ampliación B
usa la autorización posterior para migrar únicamente `horbyta_development`. El ensayo de restauración usa PostgreSQL local desechable:
comprueba registros, checksums y restricciones, pero no acredita una restauración
del bucket o de archivos reales de una VPS. Ese ensayo corresponde a su entorno.

### Verificaciones repetibles

`tests/scope_control.rs` crea un esquema aislado, instala el esquema anterior y
ensaya antecedentes en los siete estados, incluyendo moneda desconocida. Comprueba
idempotencia de la migración, inmutabilidad, roles, conflictos y carreras reales
entre dos conexiones; también rollback de efectos, conservación, modalidades de
cobro, tiempo no facturable y recuperación del outbox. El esquema se elimina al
terminar, también si falla una aserción. La conexión necesita `CREATE SCHEMA`.

El recorrido `scripts/verify-scope.mjs`, activado con `UI_VERIFY_SCOPE=1` en el
capturador, opera sobre las identidades generadas por esas pruebas. Cubre publicación
y aceptación de alcance, evaluación, clasificación, rechazo, aprobación desactualizada,
aclaraciones, factura/fecha/habilitación, reintento, pendientes, móvil, teclado,
temas claro/oscuro y desconexión. Los HTTP 409 del intento bloqueado y de la decisión
antigua son resultados esperados; no significan un fallo del recorrido.

`scripts/verify-scope-backup.py` está restringido al laboratorio local de puerto
55439: conserva un dump privado en `target/scope-tests/backup.dump`, restaura en
una base temporal, compara los registros de 18 tablas y los checksums de todas las migraciones y verifica la protección
de las revisiones. Elimina esa base restaurada al terminar. No consume el `.env`.
Las credenciales, capturas, dumps y resultados de pruebas permanecen bajo `target/`,
ignorado por Git; no publicarlos como documentación pública.

## API

Las rutas `/api/v1/scope` separan profesional y portal autenticado:

- `GET /projects/{id}` y `GET /client/projects/{id}`: alcance e historial.
- `POST /projects/{id}`: guardar/publicar con `expected_revision` y documento.
- `POST /client/projects/{id}/accept`: aceptar la revisión publicada.
- `GET /changes/{id}` y `GET /client/changes/{id}`: revisión, eventos y efectos.
- `POST /changes/{id}/comments` y equivalente del cliente: agregar aclaración.
- `GET /tasks/{id}` y `POST /tasks/{id}`: consultar/clasificar origen.
- `GET /client/pending`: decisiones pendientes persistentes.
- `POST /projects/{id}/feedback` y equivalente del cliente: comentarios/decisiones por revisión y documento.
- `POST /tasks/{id}/preparation`: justificar preparación previa al acuerdo.
- `POST /tasks/{id}/requirements`: vincular documentos de requisitos incluidos en el acuerdo.

Los endpoints de cambios existentes conservan su autenticación y añaden `version`
a la evaluación y a ambas decisiones del cliente. La evaluación añade un hito
opcional `target_milestone_id`. Un conflicto exige lectura nueva y decisión
expresa; nunca se reintenta automáticamente sobre una revisión distinta.

## Negociación inicial y documentos — ampliación consolidada

Los proyectos creados desde esta migración nacen protegidos: pueden preparar
propuestas/tareas, pero no ejecutar entregables ni registrar tiempo de ejecución
antes de aceptar el alcance. Las tareas de preparación se justifican explícitamente,
conservan su origen y su tiempo es no facturable. Los proyectos anteriores sin
acuerdo mantienen compatibilidad; publicar su alcance activa la protección. Los
que ya tenían alcance aceptado continúan protegidos. No se permiten traslados ni
desactivar la política para eludir el acuerdo. Finalizar timers de ejecución es
necesario antes de publicar; guardar un borrador histórico no retira ofertas ni
activa por sí solo la protección de un proyecto anterior.

El cliente puede aceptar, solicitar ajustes o rechazar la propuesta vigente con
una explicación conservada. Ajustes/rechazo retiran esa propuesta de la aceptación
y resuelven su aviso pendiente; el proyecto permanece en negociación. Desde una
revisión anterior puede solicitar esa versión. El freelancer prepara una nueva
publicación basada en ella, revisando precio, trabajo y fechas actuales. Nunca se
acepta una propuesta sustituida por seleccionar su número histórico. Comentarios
pueden referirse a documentos concretos y se conservan con identidad/revisión.
La aceptación del alcance original sigue siendo única; después se usan cambios.

El alcance admite cinco secciones Markdown, documentos separados y un documento
principal completo que evita repartir un archivo entre campos. Importar `.md` o
`.markdown` conserva el original UTF-8, su nombre y hash además del texto editado.
Cada documento tiene ID estable, versión calculada en el servidor, hash, función
(principal/anexo/referencia), sección opcional e identificador de requisito opcional.
Se pueden relacionar tareas con las especificaciones incluidas en el acuerdo aceptado.
Los documentos de referencia dan contexto y no amplían los entregables. La regla
de interpretación debe explicar cómo resolver contradicciones; el sistema no
interpreta automáticamente qué cláusula es más importante.

La publicación incluye únicamente los archivos seleccionados por el profesional,
con hashes de su contenido binario y función explícita de anexo incluido o referencia.
Una referencia se conserva como evidencia de contexto, sin añadir entregables. Los demás archivos del proyecto no pasan a
ser condiciones aceptadas. Imágenes o archivos locales referenciados por Markdown
requieren subir/vincular sus recursos; las imágenes remotas también se incorporan.
Los enlaces externos de consulta son referencias, no una incorporación de su
contenido mutable al acuerdo. Se usa un parser CommonMark para comprobar enlaces,
incluyendo referencias, sin confundir ejemplos en bloques de código. Se verifica
la integridad de anexos al aceptar y al descargarlos; una diferencia o ausencia
impide presentar otra versión como si fuera la publicada. Se mantienen los límites
de archivos ya existentes y hasta 50 documentos, 200 KB por documento y 1 MB entre
contenidos/originales. Publicar conserva un hash del conjunto con documentos,
archivos, tareas, condiciones económicas e hitos.

La UI muestra el estado de cada revisión por separado del acuerdo activo, permite
comparar contenido/documentos/condiciones y conserva el estado histórico al consultar.
El portal dispone de índice y búsqueda de documentos y descarga de Markdown editado
u original. La aceptación confirma el conjunto completo de documentos incluidos;
no hay aprobación parcial por requisito con efectos económicos implícitos.

### IP y despliegue

La IP es evidencia complementaria de una aceptación autenticada, no identidad
suficiente ni firma criptográfica. Se conserva IP, origen de la IP, identidad,
fecha del servidor, revisión y texto de confirmación. Los extras aprobados también
reciben su IP desde el servidor, sobrescribiendo el valor enviado en el cuerpo.
La dirección procede del socket. Solo con `TRUSTED_PROXY_IPS` se admite `X-Real-IP`
de un peer configurado. Para el Nginx de `nginx/pm-api.conf`, que sobrescribe esa
cabecera, configurar `TRUSTED_PROXY_IPS=127.0.0.1,::1` y mantener el API accesible
solo detrás de ese proxy. No se confía en cadenas `X-Forwarded-For` del visitante
ni se fabrica `127.0.0.1` cuando falta una dirección. Sin configuración de proxy,
se registra el socket y su procedencia explícita. Las aceptaciones anteriores no
reciben IP ni hashes retrospectivos inventados.

`migrate` reconoce/instala el esquema base y aplica la ampliación consolidada, comprobando sus checksums. No modificar una
migración aplicada en producción. Hacer backup consistente de base y objetos,
ensayar restauración y publicar backend/Quasar juntos. La modificación directa
de `horbyta_development` está autorizada por el usuario; esta autorización no
incluye aplicar migraciones en producción. El respaldo privado previo a 002 se
conserva en `target/scope-tests/development-before-002.dump`. La reconstrucción
consolidada posterior guarda su respaldo completo en
`target/development-seed/before-bootstrap-reset.dump`. El comando de seed y los
datos de acceso están documentados en README.

### Conversación del proyecto

La conversación conserva la negociación del proyecto y el contexto de cada mensaje. Sus mensajes
no cambian entregables, condiciones ni aceptación; un archivo enviado aquí no
se convierte en anexo contractual. Los adjuntos enviados se conservan como
historial y tienen permisos del proyecto. Audio no admitido en conversaciones;
el visor general de adjuntos conserva su soporte anterior.

`GET /api/v1/scope/projects/{id}/conversation` devuelve como máximo
100 mensajes y `has_older`/`has_newer`. Sin cursor devuelve los últimos; `before`
o `after` usa el ID de un mensaje de ese mismo proyecto. No acepta
ambos cursores juntos. El portal usa `/scope/client/projects/...`. La interfaz
sustituye la ventana completa al avanzar/retroceder; no acumula páginas ni
borra registros históricos. Scroll interno de 240px hasta min(55vh, 600px).

`POST .../conversation/files` recibe multipart, máximo 10 archivos y
50 MiB combinados, sujeto a cuota. `POST .../conversation` admite `attachment_ids`,
referencias autorizadas y mensajes sin texto con archivos. Las decisiones explícitas
siguen usando sus rutas de aceptación/feedback; ajustes/rechazos requieren explicación.
Los links HTTP(S) son clicables y el texto se escapa; no se interpreta HTML.

El profesional reutiliza `/ws`; el cliente usa `/ws/scope/client`, autenticado
con su cookie y validación de origen. Los eventos se filtran por propietario y
cliente y se comprueban los permisos vigentes antes de enviarlos. Tras reconectar
se consulta la ventana actual. Los eventos no llevan archivos ni historial
completo. No hay un segundo socket profesional que consuma sus conexiones.

Los avisos del chat se silencian si la pestaña de conversación de ese proyecto está
abierta y el navegador está visible y enfocado. Los mensajes solo se reconocen como
leídos al alcanzar el final visible. Consultar historial conserva pendientes los
mensajes nuevos. El portal tiene campana e inbox persistente agrupado por proyecto.

Aplicar la migración 005 con `migrate` antes de arrancar el backend actualizado.
Las rutas anteriores por revisión se mantienen por compatibilidad; la interfaz usa
las rutas del proyecto y no crea conversaciones separadas.

### Notificaciones de mensajes

Los avisos nuevos incluyen message_id y enlazan al mensaje dentro de la conversación única del proyecto.
El clic carga una ventana `around=<UUID>` de hasta 100 mensajes, centra y resalta
el mensaje, sin abrir su texto en un diálogo ni tratar metadatos como error.
`around`, `before`, `after` y `notification` son cursores excluyentes.

El cuerpo del aviso solo incluye texto de hasta 60 caracteres Unicode con espacios
normalizados. Los mensajes más largos no se incluyen en el aviso. La interfaz
aplica también esa regla a notificaciones antiguas y limita los previews generales.
Para avisos anteriores sin message_id, `notification=<UUID>` busca una coincidencia
única del mismo propietario, proyecto, revisión y texto; sin coincidencia única abre
la conversación sin señalar un mensaje arbitrario. Esta recuperación solo admite
el rol profesional, propietario de la notificación. La migración 005 conserva las referencias anteriores.

## Conversación única por proyecto (migración 005)

La conversación del proyecto sustituye las conversaciones separadas por revisión.
Las solicitudes abren el mismo historial, con referencia explícita a su registro.
`project_conversation_timeline` reúne mensajes antiguos, publicaciones, aceptación,
rechazo, ajustes y estados públicos de solicitudes. Borradores privados no aparecen.
Cambiar el selector de revisiones no genera un evento. Cada consulta devuelve hasta
100 elementos por cursor; navegar a un mensaje conserva su ID histórico.

Los mensajes no autorizan trabajo. «Crear solicitud de cambio» prepara una petición
con el mensaje original conservado en `conversation_change_sources`; se mantiene
el flujo existente de evaluación, consentimiento y efectos transaccionales.
La acción solo aparece después de aceptar el alcance inicial.

`project_conversation_reads` conserva cursores privados por proyecto e identidad.
Solo se reconocen al llegar al final visible de la conversación con la ventana enfocada.
Las referencias a recursos se validan en backend, incluida su visibilidad para el
cliente destinatario. Adjuntos limitados a diez; audio no admitido.

Las novedades refrescan badges y campana del cliente mediante WebSocket, y la
consulta persistente del inbox restaura el estado al reconectar. La lectura de una
identidad no revela confirmaciones públicas de lectura ni presencia.

API activa: `GET /scope/conversation/inbox`, `GET/POST /scope/projects/{id}/conversation`,
`POST .../conversation/read`, `GET .../conversation/resources?search=`,
`POST .../conversation/files` y `POST .../conversation/{message}/change`.
El portal usa el prefijo `/scope/client`. Enter envía; Shift+Enter añade una línea;
la composición IME no dispara envíos.

Comprobación de extremo a extremo: `UI_VERIFY_PROJECT_CHAT=1 python3 scripts/verify-scope-browser.py`
usando el esquema aislado creado por `scope_control`. En esta laptop se ejecuta dentro
de una cuota nativa `systemd-run --user --scope -p CPUQuota=50% -p MemoryMax=1024M -p MemorySwapMax=0`.
Incluye ambos roles, websocket, historial limitado, referencias, formalización y lecturas persistentes.

### Menciones inline y justificativas

Escribir `@` abre sugerencias de proyectos, hitos y solicitudes de cambio de la misma
relación con el cliente. Un proyecto sin cliente solo permite recursos de ese proyecto.
El portal excluye borradores privados y solicitudes de otro destinatario. Cada opción
incluye su tipo a la derecha. `#` busca archivos del proyecto: el cliente solo puede
elegir archivos previamente compartidos mediante alcance publicado o conversación.
La API distingue ambas búsquedas con `resources?search=...&kind=resources|files`.

La selección inserta un chip dentro del editor, sin botón de eliminación, y cierra
las sugerencias. Backspace/Delete elimina la referencia completa; el texto alrededor
sigue siendo editable. Sus posiciones Unicode y permisos se verifican en backend.
Las menciones enviadas son enlaces dentro del párrafo. Las de archivos abren el visor
existente y conservan el archivo como adjunto inmutable, sin incorporarlo al contrato.
No se admiten archivos de audio. Las referencias históricas conservan sus enlaces.

Navegar entre recursos carga el registro del nuevo destino. Las referencias a proyectos
activan Alcance y las de solicitudes activan Acuerdo. El cambio de registro mantiene
la advertencia de cambios sin guardar; cambiar únicamente la pestaña no reinicia la vista.

Rechazos y solicitudes de ajustes conservan su separador de evento, pero su
justificativa y adjuntos se muestran en una burbuja con autor y hora. Cada elemento
mantiene un único ID navegable y no duplica el límite de 100 registros.
El índice del proyecto activa Alcance antes de enfocar cualquier sección.

## Correcciones y alcance consolidado (migración 006)

«Nueva revisión» sigue disponible después de aceptar un acuerdo. El modal recomienda
solicitud de cambio cuando cambian entregables, precio o fechas. La alternativa de
corrección exige un motivo conservado en el documento y una nueva aceptación del
cliente. Rechazar o pedir ajustes a esa propuesta conserva el acuerdo vigente.
También pueden guardarse y recuperar borradores de corrección.

Publicar usa `document.agreement_revision` para identificar el acuerdo vigente y
`document.amendment_reason` para declarar el motivo. `expected_revision` conserva
su función de detectar otras publicaciones/borradores. El backend y los triggers
validan ambas versiones; un cambio concurrente devuelve 409. La nueva aceptación
conserva las firmas históricas, los vínculos de tareas y los suplementos aprobados.
Los botones al pie del Acuerdo y los atajos usan los mismos permisos y decisiones.

Aprobar una solicitud incorpora automáticamente una revisión `consolidation` en la
misma transacción que el suplemento y las fechas. Copia el documento, archivos,
trabajo base y condiciones económicas del acuerdo vigente. `approved_changes`
conserva las ofertas añadidas, importes, fechas, aprobación, IP e ID de factura;
`previous_accepted_revision` conserva el vínculo al acuerdo anterior y
`source_change_id` identifica el cambio añadido. La interfaz muestra los cambios
incluidos con sus documentos y enlaces. El historial del chat registra la consolidación
después de la aprobación. No se reescribe el Markdown ni se mezcla el suplemento con
el precio base para facturarlo otra vez.

El estado vigente apunta a la revisión consolidada, pero `accepted_at` y `accepted_by`
permanecen vacíos en ella: no se fabrica una firma del documento entero. El consentimiento
procede del acuerdo anterior y del cambio aprobado, ambos conservados. La UI usa
«Consolidado», no una nueva etiqueta de aceptación. No se vuelve a pedir aceptar el extra.
Dos ofertas independientes sobre el mismo documento pueden consolidarse sucesivamente.
Aceptar una corrección del documento obliga a reevaluar ofertas anteriores aún pendientes.
Una consolidación también deja obsoleta una corrección pendiente basada en el acuerdo
anterior: su documento se conserva y puede prepararse una revisión actualizada.

La migración 006 incorpora cambios históricos no legacy con efectos verificables;
no cambia firmas, facturas ni pagos anteriores. `schema.sql` y migraciones 001–005
permanecen intactos. Aplicar `cargo run --locked --bin migrate` antes de reiniciar.
Pruebas en esquema aislado: `cargo test --locked --test scope_amendments -- --ignored --test-threads=1`.

## Entrega por eventos (migración 007)

La cola scope_delivery conserva las decisiones antes de generar su notificación.
El worker anterior la consultaba cada cinco segundos aun vacía. Ahora una conexión
PostgreSQL dedicada escucha un canal por esquema, sin contenido privado. El trigger
avisa al insertar una entrega o adelantar su fecha; PostgreSQL publica el aviso solo
al confirmar la transacción. No se modifica ninguna migración anterior ni schema.sql.

La aplicación suscribe antes de recuperar pendientes y repite la recuperación tras
reconectar: NOTIFY es una señal, no el almacenamiento durable. La entrega y su marca
siguen siendo atómicas. Una cola vacía espera la señal sin temporizador ni consultas;
una entrega futura o fallida mantiene un temporizador hasta el próximo intento.
La conexión de escucha ocupa una conexión adicional, separada del pool de consultas,
y requiere conexión de sesión PostgreSQL (no transaction pooling para LISTEN).
Esta modificación conserva los buses websocket del proceso; no añade distribución
de esos buses entre réplicas del backend.

El heartbeat websocket del cliente solo envía Ping. Se conserva expiración de sesión
y autorización contra la base antes de entregar cada evento; una cuenta desactivada
no recibe datos nuevos aunque el socket permanezca conectado hasta expirar.
No se suprimieron logs SQL para ocultar el sondeo.

Aplicar cargo run --locked --bin migrate antes de reiniciar el backend. Las pruebas
scope_delivery usan un esquema aislado: commit/rollback, recuperación, programación,
fallo/reintento y observación de query_start durante reposo desde otra conexión.
Referencia: https://www.postgresql.org/docs/current/sql-listen.html
