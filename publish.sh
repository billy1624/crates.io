cp dist/index.html dist/404.html

cd ~/Projects/billy1624.github.io
find -not -path "./.git/*" -not -name ".git" -not -name ".gitignore" -delete

cd ~/Projects/rustacean.info
find -not -path "./.git/*" -not -name ".git" -not -name ".gitignore" -delete

cp -a ~/Projects/crates.io/dist/* ~/Projects/billy1624.github.io/
cp -a ~/Projects/crates.io/dist/* ~/Projects/rustacean.info/
