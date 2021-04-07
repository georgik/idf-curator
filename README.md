# IDF Curator

Tool for maintaining ESP-IDF envrionment.

## Quick start

Install serial drivers for ESP boards on Windows. Execute following command in PowerShell:

```
Invoke-WebRequest 'https://dl.espressif.com/dl/idf-curator/curator.exe' -OutFile .\curator.exe; .\curator.exe driver install --ftdi --silabs
```

# Commands

## Working with configuration

File stored in esp_idf.json
```
curator config get
curator config get --property gitPath
curator config get --property python --idf-path "C:/esp/"
curator config add --idf-version "v4.2" --idf-path "C:/esp/" --python "C:/python/python.exe"
curator config add --name idf --idf-version "v4.2" --idf-path "C:/esp/" --python "C:/python/python.exe"
curator config rm id
```

### Working with installations of ESP-IDF
```
curator idf install
curator idf install --idf-version "master" --installer "G:\idf-installer\build\esp-idf-tools-setup-online-unsigned.exe"
curator idf uninstall
curator idf reset --path "G:\esp-idf"
```

### Working with Antivirus

```
curator antivirus get --property displayName
curator antivirus register --path "C:\....exe"
curator antivirus unregister --path "C:\....exe"
```


### Working with drivers

```
curator driver get
curator driver get --property DeviceID
curator driver get --property DeviceID --missing
```

Run in elevated shell - requires Administrator privileges.
Tools will request elevated privileges by UAC if necessary.

```
curator driver install --ftdi --silabs
```