using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using Windows.Foundation;
using Windows.Foundation.Collections;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace desktop_windows
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class SettingsPage : Page
    {
        public SettingsPage()
        {
            this.InitializeComponent();
            LoadCurrentTheme();
        }

        private void LoadCurrentTheme()
        {
            if (((App)App.Current).Window?.Content is FrameworkElement rootElement)
            {
                var currentTheme = rootElement.RequestedTheme;
                
                switch (currentTheme)
                {
                    case ElementTheme.Light:
                        ThemeComboBox.SelectedIndex = 0;
                        break;
                    case ElementTheme.Dark:
                        ThemeComboBox.SelectedIndex = 1;
                        break;
                    case ElementTheme.Default:
                    default:
                        ThemeComboBox.SelectedIndex = 2;
                        break;
                }
            }
        }

        private void ThemeComboBox_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (ThemeComboBox.SelectedItem is ComboBoxItem selectedItem)
            {
                var themeTag = selectedItem.Tag.ToString();
                var window = ((App)App.Current).Window;
                
                if (window?.Content is FrameworkElement rootElement)
                {
                    switch (themeTag)
                    {
                        case "Light":
                            rootElement.RequestedTheme = ElementTheme.Light;
                            break;
                        case "Dark":
                            rootElement.RequestedTheme = ElementTheme.Dark;
                            break;
                        case "Default":
                            rootElement.RequestedTheme = ElementTheme.Default;
                            break;
                    }
                }
            }
        }
    }
}
