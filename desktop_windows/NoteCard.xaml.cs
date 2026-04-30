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
    public sealed partial class NoteCard : UserControl
    {
        public static readonly DependencyProperty NoteContentProperty =
            DependencyProperty.Register(
                nameof(NoteContent),
                typeof(string),
                typeof(NoteCard),
                new PropertyMetadata(string.Empty));

        public static readonly DependencyProperty TimeAgoProperty =
            DependencyProperty.Register(
                nameof(TimeAgo),
                typeof(string),
                typeof(NoteCard),
                new PropertyMetadata(string.Empty));

        public string NoteContent
        {
            get => (string)GetValue(NoteContentProperty);
            set => SetValue(NoteContentProperty, value);
        }

        public string TimeAgo
        {
            get => (string)GetValue(TimeAgoProperty);
            set => SetValue(TimeAgoProperty, value);
        }

        public NoteCard()
        {
            this.InitializeComponent();
        }
    }
}
