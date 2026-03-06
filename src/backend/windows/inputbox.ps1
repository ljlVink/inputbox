Add-Type -AssemblyName System.Windows.Forms;
Add-Type -AssemblyName System.Drawing;

$jsonString = $args[0];
if ([string]::IsNullOrWhiteSpace($jsonString)) {
    $stdin = [System.Console]::OpenStandardInput()
    $reader = New-Object System.IO.StreamReader($stdin,[System.Text.Encoding]::UTF8)
    $jsonString = $reader.ReadToEnd()
    $reader.Close()
}

if ([string]::IsNullOrWhiteSpace($jsonString)) {
    Write-Error "No JSON input provided.";
    exit 1;
}

$config = $jsonString | ConvertFrom-Json;

$form = New-Object System.Windows.Forms.Form;
$form.Text = $config.title;
$form.StartPosition = 'CenterScreen';
$form.FormBorderStyle = 'Sizable';
$form.MaximizeBox = $true;
$form.MinimizeBox = $false;
$form.TopMost = $true;

$label = New-Object System.Windows.Forms.Label;
$label.Text = $config.prompt;
$label.Location = New-Object System.Drawing.Point(15, 15);
$label.AutoSize = $true;
$label.MaximumSize = New-Object System.Drawing.Size(350, 0);
$form.Controls.Add($label);

$form.PerformLayout();
$labelBottom = $label.Location.Y + $label.Height;

$textBox = New-Object System.Windows.Forms.TextBox;
$textBox.Location = New-Object System.Drawing.Point(15, ($labelBottom + 10));
$textBox.Text = $config.default;

if ($config.mode -eq "multiline") {
    $textBox.Multiline = $true;
    $textBox.Size = New-Object System.Drawing.Size(350, 150);
    $textBox.WordWrap = $config.auto_wrap;

    if ($textBox.WordWrap) {
        $textBox.ScrollBars = 'Vertical';
    } else {
        $textBox.ScrollBars = 'Both';
    }

    $textBox.AcceptsReturn = $true;
} else {
    $textBox.Multiline = $false;
    $textBox.Size = New-Object System.Drawing.Size(350, 25);

    if ($config.mode -eq "password") {
        $textBox.UseSystemPasswordChar = $true;
    }
}
$form.Controls.Add($textBox);

$textBoxBottom = $textBox.Location.Y + $textBox.Height;

$cancelButton = New-Object System.Windows.Forms.Button;
$cancelButton.Size = New-Object System.Drawing.Size(75, 25);
$cancelButton.Location = New-Object System.Drawing.Point(210, ($textBoxBottom + 15));
$cancelButton.Text = $config.cancel_label;
$cancelButton.DialogResult =[System.Windows.Forms.DialogResult]::Cancel;
$form.Controls.Add($cancelButton);

$okButton = New-Object System.Windows.Forms.Button;
$okButton.Size = New-Object System.Drawing.Size(75, 25);
$okButton.Location = New-Object System.Drawing.Point(290, ($textBoxBottom + 15));
$okButton.Text = $config.ok_label;
$okButton.DialogResult =[System.Windows.Forms.DialogResult]::OK;
$form.Controls.Add($okButton);

$form.CancelButton = $cancelButton;

if ($config.mode -ne "multiline") {
    $form.AcceptButton = $okButton;
}

$form.ClientSize = New-Object System.Drawing.Size(380, ($cancelButton.Location.Y + $cancelButton.Height + 15));

if ($config.mode -eq "multiline") {
    $textBox.Anchor = 'Top, Bottom, Left, Right';
} else {
    $textBox.Anchor = 'Top, Left, Right';
}

$cancelButton.Anchor = 'Bottom, Right';
$okButton.Anchor = 'Bottom, Right';

$size = $form.ClientSize;
if ($null -ne $config.width) {
    $size.Width = [int]$config.width;
}
if ($null -ne $config.height) {
    $size.Height = [int]$config.height;
}
$form.ClientSize = $size;

$form.Add_Shown({
    $textBox.Select();

    $scrollToEnd = $config.scroll_to_end;

    if ($scrollToEnd) {
        $textBox.SelectionStart = $textBox.Text.Length;
        $textBox.SelectionLength = 0;
        $textBox.ScrollToCaret();
    } else {
        $textBox.SelectionStart = 0;
        $textBox.SelectionLength = 0;
    }
});

$result = $form.ShowDialog();

if ($result -eq[System.Windows.Forms.DialogResult]::OK) {
    $res = $textBox.Text;
    if ($null -ne $res) {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($res);
        $stdout =[System.Console]::OpenStandardOutput();
        $stdout.Write($bytes, 0, $bytes.Length);
        $stdout.Flush();
    }
} else {
    exit 1;
}
