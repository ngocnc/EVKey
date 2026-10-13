@echo off
rem Builds tools\difftest\out\harness.exe from the ORIGINAL OpenKey engine sources.
rem Files under third_party\ are compiled unmodified.
setlocal
set "HERE=%~dp0"
set "ENGINE=%HERE%..\..\third_party\OpenKey\Sources\OpenKey\engine"
set "OUT=%HERE%out"
if not exist "%OUT%" mkdir "%OUT%"

call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 (echo vcvars64.bat failed & exit /b 1)

pushd "%OUT%"
cl /nologo /utf-8 /EHsc /std:c++17 /O2 /W1 /D_CRT_SECURE_NO_WARNINGS /D_SILENCE_CXX17_CODECVT_HEADER_DEPRECATION_WARNING /D_HAS_STD_BYTE=0 /I"%ENGINE%" /Fe:harness.exe "%HERE%harness.cpp" "%ENGINE%\Engine.cpp" "%ENGINE%\Vietnamese.cpp" "%ENGINE%\Macro.cpp" "%ENGINE%\SmartSwitchKey.cpp" "%ENGINE%\ConvertTool.cpp"
set "RC=%errorlevel%"
popd
if not "%RC%"=="0" (echo harness build failed & exit /b 1)
echo built "%OUT%\harness.exe"
