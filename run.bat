@echo off
if not exist node_modules call npm install
npm run tauri dev
