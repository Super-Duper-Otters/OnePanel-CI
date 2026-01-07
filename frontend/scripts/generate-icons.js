
import sharp from 'sharp';
import pngToIco from 'png-to-ico';
import fs from 'fs';
import path from 'path';

const svgPath = path.resolve('public', 'logo.svg');
const backendAssetsDir = path.resolve('../backend/assets');

if (!fs.existsSync(backendAssetsDir)) {
    fs.mkdirSync(backendAssetsDir, { recursive: true });
}

async function generate() {
    console.log('Generating icons...');

    // 1. Generate PNG for Tray (System Tray usually needs small sizes, but we can give it 256x256 and let it scale or provide multiple)
    // Standard tray icon: 32x32 usually enough, but let's do high res
    const pngPath = path.join(backendAssetsDir, 'icon.png');
    await sharp(svgPath)
        .resize(256, 256)
        .png()
        .toFile(pngPath);
    console.log(`Generated ${pngPath}`);

    // 2. Generate ICO for Windows Exe
    const icoPath = path.join(backendAssetsDir, 'icon.ico');

    // Create buffers for different sizes
    const sizes = [16, 32, 48, 64, 128, 256];
    const buffers = await Promise.all(sizes.map(size =>
        sharp(svgPath).resize(size, size).png().toBuffer()
    ));

    const icoBuf = await pngToIco(buffers);
    fs.writeFileSync(icoPath, icoBuf);
    console.log(`Generated ${icoPath}`);
}

generate().catch(err => {
    console.error(err);
    process.exit(1);
});
