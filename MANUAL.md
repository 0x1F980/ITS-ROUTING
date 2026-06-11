# Hydra-ITS: Shadow Network Manual, Documentation, and Mathematical Proofs
### Modulus $\mathbb{Z}_{251}$ — Information-Theoretic Secrecy (ITS) Reference

---

## 1. System Arkitektur & Oversigt
Hydra-ITS er et højtydende, bare-metal skyggenetværk designet til at levere **Information-Theoretic Secrecy (ITS)**. I modsætning til konventionelle netværk (såsom Tor eller Nym), som er sårbare over for fremtidige kvantecomputere, timing-analyser og supercomputere med enorm beregningskraft, er Hydra-ITS beskyttet af de uomgængelige love for ren algebra og sandsynlighedsteori.

Systemet består af fem modulære, tæt integrerede kryptografiske lag:

```
+------------------------------------------------------------+
|            Passiv Entropi-Parasitisme (PEP)                |  <- Absolut anonymitet og 0 afsender-skyldighed
+------------------------------------------------------------+
                             |
+------------------------------------------------------------+
|        SSS-Chained Perfect Secrecy Trapdoors (SCPST)       |  <- Dynamiske, støj-resistente geometriske tunneler
+------------------------------------------------------------+
                             |
+------------------------------------------------------------+
|             Morphic Network Coding (MNC)                   |  <- Blind, algebraisk pakke-blanding på routere
+------------------------------------------------------------+
                             |
+------------------------------------------------------------+
|         Shamir's Secret Sharing (base-251 SSS)             |  <- Tabssikker fragmentering af data
+------------------------------------------------------------+
                             |
+------------------------------------------------------------+
|             Konstant-tids aritmetik i Z_251                |  <- Hardware sidekanals- og timing-modstand
+------------------------------------------------------------+
```

---

## 2. Matematisk Bevisførelse af Uangribelighed

### Bevis 1: Shannon’s Perfect Secrecy på SSS-fragmenter
Claude Shannon beviste i 1949, at et kryptosystem har **Perfect Secrecy** (perfekt hemmelighed), hvis og kun hvis den a posteriori sandsynlighedsfordeling af en klartekst $M$, efter at have observeret cipherteksten $C$, er nøjagtig lig med a priori sandsynlighedsfordelingen af $M$:
$$P(M \mid C) = P(M)$$

I vores implementering (`hydra_sss.rs`) anvender vi Shamir's Secret Sharing over det endelige felt $\mathbb{Z}_{251}$. For at dele en hemmelig byte $S \in \mathbb{Z}_{251}$ med en tærskel på $k$, genererer vi et tilfældigt polynomium af grad $k-1$:
$$P(x) = S + a_1 \cdot x + a_2 \cdot x^2 + \dots + a_{k-1} \cdot x^{k-1} \pmod{251}$$
Hvor koefficienterne $a_1, a_2, \dots, a_{k-1}$ vælges fuldstændig uniformt og uafhængigt fra den fysiske entropi-kilde (TRNG) over feltet $\mathbb{Z}_{251}$.

#### Det algebraiske bevis:
Antag, at en angriber (Eve) besidder fuld kontrol over $k-1$ routere og opsnapper $k-1$ shares:
$$\mathcal{E} = \{(x_1, y_1), (x_2, y_2), \dots, (x_{k-1}, y_{k-1})\}$$
Hvor $y_i = P(x_1) \pmod{251}$ og alle $x_i$ er unikke og ikke-nul ($x_i \ne 0$).

For at rekonstruere det hemmelige polynomium $P(x)$ og dermed finde hemmeligheden $S = P(0)$, skal Eve løse det lineære ligningssystem af formen:
$$\begin{pmatrix} 1 & x_1 & x_1^2 & \dots & x_1^{k-1} \\ 1 & x_2 & x_2^2 & \dots & x_2^{k-1} \\ \vdots & \vdots & \vdots & \ddots & \vdots \\ 1 & x_{k-1} & x_{k-1}^2 & \dots & x_{k-1}^{k-1} \end{pmatrix} \begin{pmatrix} S \\ a_1 \\ a_2 \\ \vdots \\ a_{k-1} \end{pmatrix} = \begin{pmatrix} y_1 \\ y_2 \\ \vdots \\ y_{k-1} \end{pmatrix} \pmod{251}$$

Dette system har $k-1$ ligninger og $k$ ubekendte ($S$ og de $k-1$ koefficienter). Matricen (en reduceret Vandermonde-matrix) har fuld række-rang $k-1$. 

For ethvert tænkeligt gæt på hemmeligheden $S' \in \mathbb{Z}_{251}$, kan vi tilføje ligningen $P(0) = S'$ til systemet. Dette skaber en kvadratisk $k \times k$ Vandermonde-matrix, som har en determinant forskellig fra nul ($\det(V) \ne 0$), da alle $x_i$ er distinkte og ikke-nul. Der findes derfor **præcis én unik løsning** for koefficienterne $(a_1', a_2', \dots, a_{k-1}')$ for *hvert eneste mulige valg* af $S'$.

Da der er nøjagtig $251^{k-1}$ mulige polynomier af grad $k-1$, som passer på de opsnappede $k-1$ punkter, og da alle disse polynomier forekommer med nøjagtig samme uniforme sandsynlighed:
$$P(\mathcal{E} \mid S = s) = \frac{1}{251^{k-1}} \quad \forall s \in \mathbb{Z}_{251}$$
Dette medfører ifølge Bayes' teorem:
$$P(S = s \mid \mathcal{E}) = \frac{P(\mathcal{E} \mid S = s) \cdot P(S = s)}{\sum_{j=0}^{250} P(\mathcal{E} \mid S = j) \cdot P(S = j)} = P(S = s)$$

**Q.E.D.:** Sandsynligheden for at hemmeligheden er $s$, efter at have set de opsnappede shares, er nøjagtig den samme som før. Selvom Eve har uendelig regnekraft, kan hun aldrig gætte værdien med en sandsynlighed højere end $\frac{1}{251}$.

---

### Bevis 2: Information-Theoretic Blindness under Morphic Network Coding (MNC)
Når to uafhængige pakker (Alices pakke $P_A$ og Claires pakke $P_C$) passerer gennem en intermediær VPS-node (`routing.rs`), blandes de blindt uden dekryptering vha. lineære skalar-faktorer $c_1, c_2 \in \mathbb{Z}_{251}$ valgt af noden:
$$P_{\text{morphed}} = c_1 \cdot P_A + c_2 \cdot P_C \pmod{251}$$

Antag, at Eve beslaglægger denne router og forsøger at dekryptere Alices originale pakke $P_A$. Hun kender $P_{\text{morphed}}$ og de lokale parametre $c_1, c_2$.
Hver byte $y_{\text{morphed}}$ i pakken er defineret ved ligningen:
$$y_{\text{morphed}} = c_1 \cdot y_A + c_2 \cdot y_C \pmod{251}$$

Dette er en enkelt ligning med to ubekendte feltelementer ($y_A, y_C$). For ethvert gæt, Eve foretager på Alices oprindelige værdi $y_A' \in \mathbb{Z}_{251}$, findes der en entydig, matematisk perfekt værdi for Claires del $y_C'$:
$$y_C' = c_2^{-1} \cdot (y_{\text{morphed}} - c_1 \cdot y_A') \pmod{251}$$

Da alle kombinationer er lige gyldige, er systemet **algebraisk underbestemt**. Eve besidder absolut ingen statistiske holdepunkter for at skille Alices data fra Claires.

---

### Bevis 3: Wegman-Carter OTM Integritet (Modstandsdygtighed mod MITM)
I vores `otm.rs` sikres integriteten af hver pakke vha. et Wegman-Carter One-Time MAC-tag $T$ over den maskerede y-koordinat $y_{\text{mask}}$:
$$T = K_{\text{MAC}} \cdot y_{\text{mask}} + N \pmod{251}$$
Hvor $K_{\text{MAC}}$ og noncen $N$ er engangsnøgler udledt fra de uafhængige SSS-kæder.

Hvis en aktiv angriber (Eve) forsøger at forfalske eller ændre pakken til en modificeret værdi $y_{\text{mask}}' \ne y_{\text{mask}}$, skal hun generere et gyldigt tag $T'$:
$$T' = T + K_{\text{MAC}} \cdot (y_{\text{mask}}' - y_{\text{mask}}) \pmod{251}$$

For at beregne denne ændring korrekt, skal hun kende $K_{\text{MAC}}$. Men da hun kun har observeret ét enkelt tag $T$ for den hemmelige nøgle $K_{\text{MAC}}$ og den hemmelige nonce $N$, er systemet igen underbestemt (én ligning, to ukendte). For ethvert gæt på $K_{\text{MAC}}$, findes der en unik værdi af $N$, som stemmer overens med hendes observation. 

Sandsynligheden for, at hun kan foretage en succesfuld Man-in-the-Middle modifikation uden at blive opdaget, er præcis:
$$P_{\text{forgery}} = \frac{1}{251} \approx 0.398\%$$
Dette er matematisk bevist uafhængigt af hendes beregningsmæssige ressourcer.

---

## 3. Latency, Flaskehalse & Optimeringer for Web-hosting

Når man hoster en hjemmeside over Hydra-ITS skyggenetværket, er der specifikke tekniske faktorer, som dikterer svartiderne (latency):

| Flaskehals | Typisk Latency-påvirkning | Årsag | Optimering |
| :--- | :--- | :--- | :--- |
| **Chaff Queue Ticks** | 100 ms – 500 ms per hop | Pakker holdes i buffer for at udjævne timing-mønstre | Reducer `tick_rate_ms` til 50 ms (kræver mere båndbredde) |
| **SSS Base-251 Splitting** | 1 ms – 5 ms per side | Generering af tilfældige polynomier for hver 1 byte | Pipeline beregninger i Rust vha. SIMD-instruktioner |
| **Lagrange-interpolation** | 2 ms – 15 ms per side | Reconstructions-loops på modtagerens computer | Forudberegn Lagrange-koefficienter $\ell_i(0)$ asynkront |
| **Båndbredde Multiplikation** | Afhænger af WAN-link | $n$- shares sendes parallelt over netværket | Kør med lav tærskel ($k=2, n=3$) for hurtigere transit |

### Formel for Latency-estimat på en webside hentet over $H$ hop:
$$\text{Latency} \approx \sum_{i=1}^{H} \left( \text{WAN\_RTT}_{i} + \text{Tick\_Delay}_{i} \right) + T_{\text{SSS}} \times \text{Webside\_Størrelse}$$

---

## 4. Streaming af Video over Netværket
Selvom netværket er optimeret til ultra-sikker, lavbånds-kommunikation, er video-streaming teknisk muligt, hvis man anvender de rette komprimerings- og system-konfigurationer:

1. **AV1 / H.265 Ultra-komprimering:**
   Traditionel streaming kræver 5-10 Mbps. Ved at kode videoen i **AV1** med en opløsning på 360p eller 480p ved 15 frames per sekund, kan vi bringe videostrømmen ned på under **200 kbps** uden tab af afgørende synlig information.
2. **Erasure Coding Fordelen:**
   Da video-strømmen er splittet vha. Shamir's Secret Sharing (f.eks. $k=3, n=5$), behøver modtageren kun at modtage shares fra de **3 hurtigste noder** for at afspille videoframen. Systemet fjerner dermed buffering forårsaget af midlertidigt langsomme noder ("tail-latency" elimineres).
3. **Adaptive Security Levels:**
   For tunge mediefiler kan man køre i en hybrid-tilstand, hvor kun kontrolsignaler og identitetsudveksling beskyttes med fuld ITS-chaff, mens selve videodataen sendes kryptered i en parallel, hurtig SCPST-tunnel uden chaff-forsinkelser.

---

## 5. Opsætningsmanual & CLI Reference

Det fulde system kan styres og konfigureres via en ensartet CLI-grænseflade.

### Kommando 1: Start en aktiv routing-node
Dette starter en aktiv router på en VPS eller bare-metal maskine:
```bash
hydra-its start-node --config /etc/hydra-its/config.toml --port 8180 --chaff-rate 100
```

### Kommando 2: Send en krypteret webside eller besked (Alice)
Sætter gang i kryptering, base-251 splitting og distribution til routerne:
```bash
hydra-its client-send --msg "Secret Classified Intelligence Document" --dest 9 --config ./config.example.toml
```

### Kommando 3: Passiv Entropy Parasitisme (PEP) — Absolut uforudsigelig stealth-upload
Alice ændrer en ekstern telemetry-pool for at indlejre data, helt uden at kontakte modtageren:
```bash
hydra-its client-send --msg "Stealth Data" --pep --config ./config.example.toml
```

### Kommando 4: Modtag og rekonstruer data (Bob)
Bob opsamler shares fra netværket eller den passive entropi-strøm og genskaber websiden i konstant-tid:
```bash
hydra-its client-receive --pep --config ./config.example.toml
```

---

## 6. Shell Auto-completions & Unix Manpages

Vi har opbygget komplet understøttelse af dit operativsystems terminaler for at sikre en professionel og flydende brugeroplevelse.

### Shell Auto-completions
Disse filer er tilgængelige i dit arbejdsområde under `completions/` mappen:
* **Bash:** `completions/hydra-its.bash` — Kopieres til `/etc/bash_completion.d/hydra-its`
* **Zsh:** `completions/hydra-its.zsh` — Kopieres til en mappe i dit `$fpath` som `_hydra-its`
* **Fish:** `completions/hydra-its.fish` — Kopieres til `~/.config/fish/completions/hydra-its.fish`

#### Aktivering i Zsh:
```zsh
cp completions/hydra-its.zsh ~/.zsh/completion/_hydra-its
echo "fpath=(~/.zsh/completion \$fpath)" >> ~/.zshrc
echo "autoload -U compinit && compinit" >> ~/.zshrc
source ~/.zshrc
```

### Installation af Unix Manpage (Manualside)
For at læse den professionelle Unix-manualside direkte i terminalen med kommandoen `man hydra-its`, installeres den troff-formaterede manualside på følgende måde:

```bash
sudo cp man/hydra-its.1 /usr/share/man/man1/
sudo mandb
```
Derefter kan du til enhver tid kalde:
```bash
man hydra-its
```

---

## 7. Komplet Konfigurations-eksempel (`config.example.toml`)
Den medfølgende opsætningsfil styrer alle aspekter af netværkets opførsel:

```toml
# Unikt heltal i Z_251 for denne node
id = 1
port = 8180
bind_address = "0.0.0.0"

[crypto]
threshold_k = 3
total_shares_n = 5
trapdoor_x = 2
trapdoor_y = 11

[traffic]
constant_rate_chaff_enabled = true
tick_rate_ms = 100
```
