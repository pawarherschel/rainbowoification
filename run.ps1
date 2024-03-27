cargo run --release;
rm out/out.mkv;
cd out;
ffmpeg -framerate 15 -i frame_%03d.png -c:v libx264 -pix_fmt rgb24 out.mkv;
cd ..;
