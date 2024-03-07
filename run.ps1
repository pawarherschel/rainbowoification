cargo run --release;
cd out;
ffmpeg -framerate 15 -i frame_%03d.png -c:v libx264 -pix_fmt rgb24 out.mp4;
echo ^G;
