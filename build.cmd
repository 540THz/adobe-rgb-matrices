@setlocal
@REM Set %PSModulePath% to default
@REM : Inspired by 2nd method of https://stackoverflow.com/a/53464542
@set "PSModulePath=%ProgramFiles%\WindowsPowerShell\Modules;%SystemRoot%\system32\WindowsPowerShell\v1.0\Modules"
@powershell -ex bypass -f "%~dp0build.ps1"
@endlocal
@exit /b %ERRORLEVEL%
