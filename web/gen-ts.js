import path from "path";
import * as fs from "node:fs/promises"
import url from "url";
import { compileFromFile } from "json-schema-to-typescript";

const dirname = path.dirname(url.fileURLToPath(import.meta.url));

async function main() {
    let schemasPath = path.join(dirname, '..', 'schemas');
    let schemaFiles = (await fs.readdir(schemasPath)).filter((x) => x.endsWith('.json'));

    let compiledTypes = new Set()
    for (let filename of schemaFiles) {
        let filePath = path.join(schemasPath, filename);
        let schema = await compileFromFile(filePath, { bannerComment: '' });

        let eachType = schema.split("export");
        for (let type of eachType) {
            if (!type) {
                continue
            }
            compiledTypes.add('export ' + type.trim())
        }
    }

    let output = Array.from(compiledTypes).join('\n\n');
    let outputPath = path.join(dirname, 'api_types.ts');

    await fs.writeFile(outputPath, output);
    console.log(`Wrote Typescript types to ${outputPath}`);
}

main().catch((e) => {
    console.error(e)
    process.exit(1)
})