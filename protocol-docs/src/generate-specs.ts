import { compileFromFile } from "json-schema-to-typescript";
import { Options as PrettierOptions } from "prettier";
import * as fs from "fs";
import camelcase = require("camelcase");

const SCHEMA_EXTENSION: string = ".schema.json";
const BUFFER_ENCODING: BufferEncoding = "utf-8";
const SCHEMAS_DIR_PATH: fs.PathLike = "../shared/schemas/";
const SPECS_FILE_PATH: fs.PathLike = "./README.md";
const SPECS_HEADER_FILE_PATH: fs.PathLike = "./SPECS_HEADER.md";
const PRITTIER_RC_FILE_PATH: fs.PathLike = "./.prettierrc";
const EXCLUDE_FILES: string[] = ["message.schema.json"];

async function run(): Promise<void> {
    if (fs.existsSync(SPECS_FILE_PATH)) {
        fs.unlinkSync(SPECS_FILE_PATH);
    }

    let result = "";
    const specsHeader = fs.readFileSync(SPECS_HEADER_FILE_PATH, BUFFER_ENCODING).toString();
    result += specsHeader + "\n";
    const prettierRc = JSON.parse(
        fs.readFileSync(PRITTIER_RC_FILE_PATH, BUFFER_ENCODING),
        undefined,
    ) as PrettierOptions;

    const files = fs.readdirSync(SCHEMAS_DIR_PATH, BUFFER_ENCODING);
    for (const fileName of files) {
        if (
            fileName.endsWith(SCHEMA_EXTENSION, undefined) &&
            !EXCLUDE_FILES.includes(fileName, undefined)
        ) {
            const filePath = `${SCHEMAS_DIR_PATH}/${fileName}`;
            const fileContent = JSON.parse(
                fs.readFileSync(filePath as fs.PathLike, BUFFER_ENCODING),
                undefined,
            );

            result +=
                "### " +
                fileContent.kind +
                ": " +
                camelcase(fileName.split(SCHEMA_EXTENSION, undefined)[0], {
                    pascalCase: true,
                } as camelcase.Options) +
                "\n\n";
            result += fileContent.about + "\n\n";

            result += "```ts\n";
            const interfaceCode = await compileFromFile(filePath, {
                style: prettierRc,
                bannerComment: "",
            });
            result += interfaceCode.substr(interfaceCode.indexOf(" ") + 1);
            result += "```\n\n";
        }
    }

    fs.writeFileSync(SPECS_FILE_PATH, result, undefined);
}

run().catch((err) => {
    console.error(err);
});
