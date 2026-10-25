#!/usr/bin/env node
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const fs_extra_1 = __importDefault(require("fs-extra"));
const commander_1 = require("commander");
const convertCsvToXlsx_1 = require("../src/convertCsvToXlsx");
const package_json_1 = __importDefault(require("../package.json"));
commander_1.program
    .version(package_json_1.default.version, '-v, --version')
    .option('-i, --input [input]', 'Input CSV file or directory containing CSV files', 'csv')
    .option('-o, --output-dir [dir]', 'Output directory for the XLSX files', 'xlsx')
    .option('-s, --sheet-name [name]', 'Set the sheet name to be used in the XLSX files', '')
    .option('-f, --force', 'forcefully overwrite XLSX files at the output directory', false);
commander_1.program.on('--help', function () {
    console.log(``);
    console.log(`Created by: ${package_json_1.default.author.name}`);
    console.log(`Please report bugs at: ${package_json_1.default.bugs.url}`);
    console.log(`Version: ${package_json_1.default.version}`);
});
commander_1.program.parse(process.argv);
const programOptions = commander_1.program.opts();
const inputPath = path_1.default.resolve(programOptions.input);
const outputPath = path_1.default.resolve(programOptions.outputDir);
if (!fs_extra_1.default.existsSync(inputPath)) {
    console.error(`Invalid input: ${inputPath}`);
    process.exitCode = 1;
    commander_1.program.help();
}
if (!fs_extra_1.default.existsSync(outputPath)) {
    fs_extra_1.default.mkdirSync(outputPath, { recursive: true });
}
const convertFile = (sourceFile) => {
    const fileObject = path_1.default.parse(sourceFile);
    if (fileObject.ext === '.csv') {
        console.info(`Converting: ${sourceFile}`);
        const destination = path_1.default.resolve(outputPath, `${fileObject.name}.xlsx`);
        (0, convertCsvToXlsx_1.convertCsvToXlsx)(sourceFile, destination, {
            sheetName: programOptions.sheetName,
            overwrite: Boolean(programOptions.force),
        });
    }
};
if (fs_extra_1.default.statSync(inputPath).isDirectory()) {
    for (const file of fs_extra_1.default.readdirSync(inputPath)) {
        convertFile(path_1.default.resolve(inputPath, file));
    }
}
else {
    convertFile(inputPath);
}
console.info(`Conversion complete.`);
