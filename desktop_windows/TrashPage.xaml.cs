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
    public sealed partial class TrashPage : Page
    {
        public TrashPage()
        {
            this.InitializeComponent();
            LoadNotes();
        }

        private void LoadNotes()
        {
            var notes = new List<NoteItem>
            {
                new NoteItem
                {
                    NoteContent = "旧版本的需求文档，已经被新的版本替代。",
                    TimeAgo = "1天前"
                },
                new NoteItem
                {
                    NoteContent = "测试用的临时笔记，内容已经不需要了。这是一段比较长的文字，用来测试笔记卡片的最大行数显示效果，看看是否能够正确地截断多余的内容。",
                    TimeAgo = "3天前"
                },
                new NoteItem
                {
                    NoteContent = "会议记录草稿，正式版本已经保存到笔记列表中。",
                    TimeAgo = "1周前"
                }
            };

            NotesRepeater.ItemsSource = notes;
        }
    }
}
