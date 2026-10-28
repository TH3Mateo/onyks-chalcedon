# ONYKS Chalcedon - instalacja i konfiguracja

Chalcedon to aplikacja desktopowa (Windows i Linux) do zarzadzania biblioteka elementow PCB.
Laczy sie z serwerem **ONYKS Bloodstone** (web GUI, baza, SVN z bibliotekami SchLib/PcbLib).

> **Screenshoty:** w tekscie sa gotowe odwolania do plikow w `docs/img/`. Wystarczy zapisac
> zrzut pod podana nazwa, zeby pojawil sie w dokumencie.

## 1. Wymagania

| | Windows | Linux |
|---|---|---|
| System | Windows 10/11 64-bit | dystrybucja z WebKitGTK 4.1 |
| SVN (klient `svn`) | instalowany automatycznie | zaleznosc pakietu (`subversion`) |
| Sterownik ODBC PostgreSQL | instalowany automatycznie | `odbc-postgresql` / `postgresql-odbc` / `psqlodbc` |
| Tailscale (opcjonalnie) | instalowany automatycznie | osobno, jesli potrzebny |
| Serwer Bloodstone | dostepny w sieci / przez VPN | j.w. |

Potrzebujesz tez: **loginu i hasla** do serwera (to samo konto do web GUI i SVN) oraz adresu serwera.

## 2. Instalacja - Windows

1. Pobierz instalator `ONYKS Chalcedon_0.1.0_x64-setup.exe`.
2. Uruchom go i przejdz kreator. Doinstalowanie sterownika ODBC i SVN wymaga uprawnien administratora - zatwierdz monit UAC.
3. Instalator **po cichu doinstaluje brakujace skladniki**:
   - Subversion (TortoiseSVN z klientem `svn`) - jesli polecenie `svn` nie istnieje,
   - Tailscale - jesli nie jest zainstalowany,
   - sterownik PostgreSQL ODBC (psqlODBC x64) - jesli nie jest zarejestrowany.

   Moze to potrwac kilkadziesiat sekund po kliknieciu "Install"; okno kreatora nie pokazuje wtedy postepu.
4. Uruchom **ONYKS Chalcedon** z menu Start.

![Kreator instalacji](img/install-windows-wizard.png)

### Budowanie instalatora (dla deweloperow)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch-redist.ps1   # pobiera svn/tailscale/psqlodbc (.msi)
pnpm install
pnpm tauri build
```

Wynik: `src-tauri/target/release/bundle/nsis/ONYKS Chalcedon_0.1.0_x64-setup.exe`.
Pliki `.msi` w `src-tauri/redist/` nie sa w gicie.

## 3. Instalacja - Linux

Pakiety znajduja sie w katalogu `dist-linux/` (albo w wydaniu projektu).

**Debian / Ubuntu**
```bash
sudo apt install "./ONYKS Chalcedon_0.1.0_amd64.deb"
```
Zaleznosci (`subversion`, `unixodbc`, `odbc-postgresql`) zostana doinstalowane. `tailscale` jest zalecany, nie wymagany.

**Fedora / openSUSE (RPM)**
```bash
sudo dnf install "./ONYKS Chalcedon-0.1.0-1.x86_64.rpm"
```
Zaleznosci: `subversion`, `unixODBC`, `postgresql-odbc`.

**Arch Linux**
```bash
cd packaging/arch
makepkg -si
```
Sterownik `psqlodbc` jest tylko w AUR - zainstaluj go pomocnikiem AUR, jesli korzystasz z bibliotek bazy w Altium/KiCad.
Aby zbudowac z lokalnego repozytorium zamiast z GitHuba: `CHALCEDON_REPO=file:///sciezka/do/repo makepkg -si`.

Uruchomienie: z menu aplikacji albo poleceniem `onyks-chalcedon`.

## 4. Pierwsze uruchomienie

1. Otworz zakladke **Settings** (pasek ikon po lewej).
2. Wypelnij sekcje opisane nizej, zaczynajac od **Login / Password** i **Web Manager**.
3. Kliknij **Save** na dole strony. **Reset** przywraca zapisane ustawienia.

![Zakladka Settings - calosc](img/settings-overview.png)

## 5. Zakladka Settings

### 5.1 Login i haslo

Dane konta na serwerze Bloodstone. Sa uzywane do logowania w web GUI oraz do operacji SVN (Push/Pull).

| Pole | Opis |
|---|---|
| Login | nazwa uzytkownika |
| Password | haslo (pole maskowane) |

![Login i haslo](img/settings-login.png)

### 5.2 Web Manager

| Pole | Opis |
|---|---|
| Address | adres serwera, np. `http://192.168.1.10:8113` |

**Find server** przeszukuje siec lokalna (rowniez przez VPN/Tailscale) i wyswietla znalezione serwery ONYKS.
Przycisk **Use** wpisuje wybrany adres w pole Address oraz podmienia host w adresie repozytorium (sekcja Repository).
Wpis oznaczony "unknown service" to cos, co odpowiada na tym porcie, ale nie jest serwerem ONYKS.

![Web Manager i wyszukiwanie serwera](img/settings-webmanager.png)

### 5.3 Database (plik .DbLib dla Altium)

Aplikacja sama utrzymuje plik `.DbLib`: po wlaczeniu autoupdate sprawdza baze co zadany czas i dopisuje
kazda nowa tabele kategorii oraz kolumne dostawcy. Ustawienia polaczenia i reszta konfiguracji zrobionej w Altium sa zachowane.
Jesli plik nie istnieje, zostanie utworzony ze zrodlem danych ODBC `bloodstone`.

| Pole | Opis |
|---|---|
| DbLib File Path | sciezka do pliku (przycisk **Select**, okno zapisu) |
| Autoupdate | wlacza okresowa aktualizacje |
| Autoupdate Interval (in seconds) | co ile sekund sprawdzac (minimum 5) |
| Update now | aktualizacja natychmiast; ponizej widac "Last check" i "Last change" |

Wazne:
- Trzymaj plik **poza folderem repozytorium**, inaczej Push zacommituje go do SVN (`*.DbLib` jest zreszta ignorowany przez Chalcedon).
- Altium potrzebuje zrodla danych ODBC o nazwie **bloodstone** wskazujacego na baze `altium_lib` serwera
  (sterownik PostgreSQL Unicode x64). Chalcedon nie tworzy tego DSN - dodaj go raz w *Zrodla danych ODBC (64-bit)*.

![Sekcja Database](img/settings-database.png)

### 5.4 Repository (SVN)

| Pole | Opis |
|---|---|
| Address | adres repozytorium SVN, np. `http://192.168.1.10/svn/...` |
| Folder Path | lokalny folder roboczy (przycisk **Select**) |
| Autoupdate (Pull) | automatycznie pobiera zmiany z serwera co zadany interwal |
| Autoupdate Interval (in seconds) | interwal pobierania |
| Auto-push on file change | automatycznie commituje, gdy dowolny plik w folderze (rekursywnie) sie zmieni |

Oba tryby mozna wlaczyc jednoczesnie; w danej chwili wykonywana jest tylko jedna operacja SVN.
Przyciski **Push** i **Pull** na pasku ikon dzialaja zawsze, niezaleznie od autoupdate.
Pod polami widac czas ostatniego Pull i Push oraz ewentualny blad.

Zmiana IP serwera (np. siec lokalna <-> VPN) jest obslugiwana automatycznie: Chalcedon wykonuje `svn relocate` na biezacy adres.
Pliki `*.DbLib`, `*.tmp`, `Thumbs.db`, `desktop.ini` sa zawsze ignorowane.

![Sekcja Repository](img/settings-repository.png)

### 5.5 Tailscale (opcjonalnie)

Pozwala laczyc sie z serwerem przez VPN Tailscale bez recznego logowania.

| Pole | Opis |
|---|---|
| Connect to Tailscale on startup | uruchamia klienta i loguje do sieci przy starcie aplikacji |
| Auth Key File (.txt) | plik tekstowy z kluczem uwierzytelniajacym (przycisk **Select**) |
| Connect now | polacz od razu |

W ustawieniach zapisywana jest tylko sciezka do pliku, nigdy sam klucz.

![Sekcja Tailscale](img/settings-tailscale.png)

## 6. Sprawdzenie, czy dziala

1. W **Settings** kliknij **Find server** albo wpisz adres recznie, nastepnie **Save**.
2. Pierwsza zakladka paska ikon otwiera web GUI serwera (zaloguj sie tym samym kontem).
3. **Pull** powinien pobrac biblioteki do wybranego folderu.
4. W **Database** kliknij **Update now** - w "Last change" pojawi sie czas, a plik `.DbLib` zostanie utworzony/uzupelniony.

![Web GUI w oknie aplikacji](img/web-gui.png)

## 7. Rozwiazywanie problemow

| Objaw | Co sprawdzic |
|---|---|
| "No server found" | czy komputer jest w tej samej sieci/VPN; port serwera (domyslnie 8113); firewall |
| Blad Push/Pull | login/haslo, adres repozytorium, czy `svn --version` dziala w terminalu |
| Altium nie widzi tabel | DSN ODBC `bloodstone` (64-bit) wskazuje na `altium_lib`; zainstalowany sterownik psqlODBC |
| Brak polaczenia po zmianie sieci | poczekaj na kolejny Pull (adres repozytorium zostanie przeniesiony `svn relocate`) lub uzyj **Find server** + **Use** |
