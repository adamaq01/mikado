

# Mikado

Pequeño hook para SDVX que envía tus puntuaciones a una instancia de Tachi mientras juegas.

## Características

- Enviar puntuaciones a una instancia de Tachi después de cada canción
- Enviar resultados de cursos a una instancia de Tachi
- Mostrar tus puntuaciones PBs de Tachi en el juego como puntuaciones de cloudlink (konaste)

## Instalación

- Descarga la última versión desde la [página de lanzamientos](https://github.com/adamaq01/mikado/releases/latest)
- Colócalo en el directorio raíz de instalación del juego (opcional: crea y edita el archivo de configuración para establecer tu clave API)
- Al iniciar el juego, inyecta la DLL en el proceso

## Consejos

- El archivo de configuración se creará en la misma carpeta que la DLL al iniciar si aún no existe
- Puedes configurar algunas opciones (como la URL de Tachi) editando el archivo `mikado.toml`
- Si estás usando Spicetools, puedes agregar la opción `-k mikado.dll` o especificar la DLL en la herramienta de configuración para
  inyectarla automáticamente al iniciar

## Licencia

MIT
