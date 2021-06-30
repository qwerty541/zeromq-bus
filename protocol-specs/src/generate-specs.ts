import { compileFromFile } from "json-schema-to-typescript";
import { Options as PrettierOptions } from "prettier";
import * as fs from "fs";
import camelcase = require("camelcase");

const BUFFER_ENCODING: BufferEncoding = "utf-8";
const SCHEMAS_DIR_PATH: fs.PathLike = "../shared/schemas/";
const SPECS_FILE_PATH: fs.PathLike = "./README.md";
const SPECS_HEADER_FILE_PATH: fs.PathLike = "./SPECS_HEADER.md";
const PRITTIER_RC_FILE_PATH: fs.PathLike = "./.prettierrc";
const EXPECTED_SCHEMA_FILE_NAME_REGEX: RegExp = new RegExp(
    "(\\d{3})(\\.{1})(.{1,999})(\\.{1})(schema{1})(\\.{1})(json{1})",
    undefined,
);

async function run(): Promise<void> {
    if (fs.existsSync(SPECS_FILE_PATH)) {
        fs.unlinkSync(SPECS_FILE_PATH);
        console.log(`Previous version of ${SPECS_FILE_PATH} successfully removed`);
    }

    let result = "";
    const specsHeader = fs.readFileSync(SPECS_HEADER_FILE_PATH, BUFFER_ENCODING).toString();
    result += specsHeader + "\n";

    console.log("Header successfully added to specification");

    const prettierRc = JSON.parse(
        fs.readFileSync(PRITTIER_RC_FILE_PATH, BUFFER_ENCODING),
        undefined,
    ) as PrettierOptions;

    const files = fs.readdirSync(SCHEMAS_DIR_PATH, BUFFER_ENCODING);
    for (const fileName of files) {
        if (EXPECTED_SCHEMA_FILE_NAME_REGEX.test(fileName)) {
            const filePath = `${SCHEMAS_DIR_PATH}/${fileName}`;
            const fileContent = JSON.parse(
                fs.readFileSync(filePath as fs.PathLike, BUFFER_ENCODING),
                undefined,
            );
            const splittedFileName = fileName.split(".", undefined);

            result +=
                "### " +
                splittedFileName[0] +
                ": " +
                camelcase(splittedFileName[1], {
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

            console.log(`"${fileName}" successfully added to specification`);
        } else {
            console.log(`"${fileName}" does not fit to regex`);
        }
    }

    fs.writeFileSync(SPECS_FILE_PATH, result, undefined);

    console.log(`Specification successfully written info "${SPECS_FILE_PATH}", work finished`);
}

run().catch((err) => {
    console.error(err);
});
