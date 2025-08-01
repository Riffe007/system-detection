# Portfolio Deployment Guide for System Monitor

## Available Executables

Your Tauri application has been successfully built with the following distribution options:

### 1. Standalone Executable (Recommended for Portfolio)
- **File**: `src-tauri/target/release/system-monitor.exe` (3.4MB)
- **Type**: Single executable file
- **Pros**: Easy to download and run, no installation required
- **Best for**: Quick demos, portfolio showcases

### 2. NSIS Installer (Professional Option)
- **File**: `src-tauri/target/release/bundle/nsis/System Monitor_0.1.0_x64-setup.exe` (1.3MB)
- **Type**: Windows installer
- **Pros**: Professional installation experience, creates start menu shortcuts
- **Best for**: Production releases, professional portfolios

## Portfolio Integration Options

### Option 1: GitHub Releases (Recommended)

1. **Create a GitHub Release**:
   ```bash
   # Create a new release on GitHub
   # Upload both executables as release assets
   ```

2. **Direct Download Links**:
   - Standalone: `https://github.com/yourusername/system-monitor/releases/latest/download/system-monitor.exe`
   - Installer: `https://github.com/yourusername/system-monitor/releases/latest/download/System-Monitor_0.1.0_x64-setup.exe`

### Option 2: Cloud Storage (Google Drive, Dropbox, etc.)

1. **Upload to Cloud Storage**:
   - Upload `system-monitor.exe` to Google Drive/Dropbox
   - Set sharing to "Anyone with link can view"
   - Use direct download links

### Option 3: Your Own Server/Website

1. **Upload to Web Server**:
   ```bash
   # Upload to your web server
   scp src-tauri/target/release/system-monitor.exe user@yourserver.com:/var/www/downloads/
   ```

## Portfolio Website Integration

### HTML Example for Portfolio

```html
<div class="project-card">
  <h3>System Monitor</h3>
  <p>A real-time system monitoring application built with Tauri, React, and Rust.</p>
  
  <div class="project-features">
    <ul>
      <li>Real-time CPU, Memory, and Network monitoring</li>
      <li>Security threat detection</li>
      <li>Performance optimization recommendations</li>
      <li>Cross-platform desktop application</li>
    </ul>
  </div>
  
  <div class="download-section">
    <h4>Download & Try</h4>
    <p>Experience the application firsthand:</p>
    
    <div class="download-buttons">
      <a href="[YOUR_DOWNLOAD_LINK]" 
         class="download-btn primary"
         download="system-monitor.exe">
        <i class="fas fa-download"></i>
        Download Standalone (3.4MB)
      </a>
      
      <a href="[YOUR_INSTALLER_LINK]" 
         class="download-btn secondary"
         download="System-Monitor-Setup.exe">
        <i class="fas fa-install"></i>
        Download Installer (1.3MB)
      </a>
    </div>
    
    <div class="system-requirements">
      <small>
        <strong>System Requirements:</strong> Windows 10/11, 4GB RAM, 100MB free space
      </small>
    </div>
  </div>
  
  <div class="tech-stack">
    <h4>Technologies Used</h4>
    <div class="tech-tags">
      <span class="tech-tag">Tauri</span>
      <span class="tech-tag">React</span>
      <span class="tech-tag">Rust</span>
      <span class="tech-tag">TypeScript</span>
      <span class="tech-tag">System Monitoring</span>
    </div>
  </div>
</div>
```

### CSS Styling Example

```css
.project-card {
  background: #ffffff;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.download-section {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 1.5rem;
  margin: 1.5rem 0;
}

.download-buttons {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
  margin: 1rem 0;
}

.download-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  text-decoration: none;
  font-weight: 600;
  transition: all 0.2s ease;
}

.download-btn.primary {
  background: #007bff;
  color: white;
}

.download-btn.primary:hover {
  background: #0056b3;
  transform: translateY(-2px);
}

.download-btn.secondary {
  background: #6c757d;
  color: white;
}

.download-btn.secondary:hover {
  background: #545b62;
  transform: translateY(-2px);
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.tech-tag {
  background: #e9ecef;
  color: #495057;
  padding: 0.25rem 0.75rem;
  border-radius: 20px;
  font-size: 0.875rem;
  font-weight: 500;
}

.system-requirements {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #dee2e6;
}
```

## Security Considerations

### Code Signing (Professional Option)
For production releases, consider code signing your executable:

```bash
# Purchase a code signing certificate
# Sign your executable
signtool sign /f certificate.pfx /p password system-monitor.exe
```

### Antivirus Considerations
- Some antivirus software may flag unsigned executables
- Consider submitting to antivirus vendors for whitelisting
- Provide source code links for transparency

## Alternative: Web Demo

If you prefer not to distribute executables, consider:

1. **Screen Recording**: Create a demo video showing the app in action
2. **Screenshots**: High-quality screenshots of all features
3. **Live Demo**: Host a web version (though limited without system access)

## Quick Setup Commands

```bash
# Copy executables to a deployment folder
mkdir portfolio-deploy
cp src-tauri/target/release/system-monitor.exe portfolio-deploy/
cp src-tauri/target/release/bundle/nsis/System\ Monitor_0.1.0_x64-setup.exe portfolio-deploy/

# Create a simple README for downloads
echo "# System Monitor - Download" > portfolio-deploy/README.md
echo "Download and run the executable to experience real-time system monitoring." >> portfolio-deploy/README.md
```

## Next Steps

1. **Choose your hosting method** (GitHub Releases recommended)
2. **Update your portfolio website** with the download links
3. **Test the download process** on a clean machine
4. **Consider adding a demo video** or screenshots
5. **Monitor download analytics** if your hosting platform provides them

## Troubleshooting

### Common Issues:
- **"Windows protected your PC"**: Normal for unsigned executables, users can click "More info" → "Run anyway"
- **Antivirus flags**: Submit to antivirus vendors for whitelisting
- **Download fails**: Check file hosting permissions and link validity

### User Instructions:
Include these instructions on your portfolio:

```
Download Instructions:
1. Click the download button above
2. Save the file to your computer
3. Right-click the downloaded file and select "Run as administrator" (if prompted)
4. If Windows shows a security warning, click "More info" then "Run anyway"
5. The application will start and begin monitoring your system
```

This setup will give your portfolio visitors a professional, downloadable experience of your system monitoring application! 