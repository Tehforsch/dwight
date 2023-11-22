cd pics && ./parsePics.sh && cd ..
pdflatex --interaction=nonstopmode -shell-escape main.tex
bibtex main
pdflatex --interaction=nonstopmode -shell-escape main.tex
pdflatex --interaction=nonstopmode -shell-escape main.tex
cat main.log | grep undefined
