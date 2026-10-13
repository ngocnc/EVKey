@echo off
rem Regenerates crates\engine\tests\data\corpus.txt (Rust generator) and
rem crates\engine\tests\data\golden.txt (original C++ engine via harness.exe).
setlocal
set "HERE=%~dp0"
set "ROOT=%HERE%..\.."
set "DATA=%ROOT%\crates\engine\tests\data"
set "CORPUS=%DATA%\corpus.txt"
set "GOLDEN=%DATA%\golden.txt"
if not exist "%DATA%" mkdir "%DATA%"

call "%HERE%build.bat"
if errorlevel 1 exit /b 1

pushd "%ROOT%"
cargo run -q -p evkey-engine --example gen_corpus > "%CORPUS%"
set "RC=%errorlevel%"
popd
if not "%RC%"=="0" (echo gen_corpus failed & exit /b 1)

"%HERE%out\harness.exe" < "%CORPUS%" > "%GOLDEN%"
if errorlevel 1 (echo harness failed & exit /b 1)

rem Validity check: one golden line per K/E/M event command in the corpus (MACRO* excluded).
for /f %%N in ('findstr /B /R "[KE]" "%CORPUS%" ^| find /c /v ""') do set /a KE=%%N
for /f %%N in ('findstr /B "M" "%CORPUS%" ^| find /c /v ""') do set /a M_ALL=%%N
for /f %%N in ('findstr /B "MACRO" "%CORPUS%" ^| find /c /v ""') do set /a MACRO_N=%%N
set /a EXPECTED=KE+M_ALL-MACRO_N
for /f %%N in ('find /c /v "" ^< "%GOLDEN%"') do set /a GOT=%%N
echo commands in corpus: %EXPECTED%  golden lines: %GOT%
if not "%EXPECTED%"=="%GOT%" (echo MISMATCH: line counts differ & exit /b 1)
echo OK: corpus and golden line counts match
