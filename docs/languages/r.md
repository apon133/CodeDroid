# 📊 R Mobile Setup & Auto-suggestions Guide

Get a full R development environment on your phone using CodeDroid and Termux.

---

## 🛠️ Step 1: Install R
Open Termux and run the following command to install the R environment:

```bash
pkg install r-base
```

Verify the installation:
```bash
R --version
Rscript --version
```

---

## 🧠 Step 2: Enable Language Support
CodeDroid runs R files using `Rscript`. Install R packages needed for your stats or data science projects inside the interactive R console:

```r
install.packages("languageserver")
```

---

## ✨ Step 3: Enable Auto-suggestions
1. Start your CodeDroid API server.
2. Open CodeDroid Web IDE and create/open any `.R` file.
3. Start typing (e.g., `print("Hello")`) and run your script directly!
