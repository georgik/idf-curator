# IDF Curator

# Commands

## Working with registry

File stored in esp_idf.json
```
curator get --property gitPath
curator get --property python --idf-path "C:/esp/"
curator add --idf-version "v4.2" --idf-path "C:/esp/" --python "C:/python/python.exe"
curator add --name idf --version "v4.2" --idf-path "C:/esp/" --python "C:/python/python.exe"
curator rm id
```

### Working with installations of ESP-IDF
```
curator inspect
curator install
curator install --idf-version "master" --installer "G:\idf-installer\build\esp-idf-tools-setup-online-unsigned.exe"
curator uninstall
```

### Working with Antivirus

```
curator antivirus --property displayName
curator antivirus register --path "C:\....exe"
curator antivirus unregister --path "C:\....exe"
```
