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
    public sealed partial class ListPage : Page
    {
        public ListPage()
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
                    NoteContent = "今天学习了WinUI3的开发框架，了解了NavigationView、Frame导航等核心组件的使用方法。",
                    TimeAgo = "1小时前"
                },
                new NoteItem
                {
                    NoteContent = "周末计划去爬山，需要准备登山杖、水壶、防晒霜等装备。天气预报显示周六是晴天，非常适合户外活动。",
                    TimeAgo = "3小时前"
                },
                new NoteItem
                {
                    NoteContent = "项目需求评审会议纪要：1. 用户反馈系统响应速度慢 2. 需要优化数据库查询 3. 增加缓存机制 4. 下周五前完成方案设计",
                    TimeAgo = "昨天"
                },
                new NoteItem
                {
                    NoteContent = "读书笔记：《代码整洁之道》- 有意义的命名、函数、注释、格式、对象和数据结构、错误处理、单元测试、类、系统等章节的核心要点总结。",
                    TimeAgo = "2天前"
                },
                new NoteItem
                {
                    NoteContent = "购物清单：牛奶、面包、鸡蛋、水果、蔬菜、洗衣液、纸巾",
                    TimeAgo = "3天前"
                },
                new NoteItem
                {
                    NoteContent = "学习Git命令：git add, git commit, git push, git pull, git branch, git merge, git rebase, git stash等常用命令的使用方法和最佳实践。",
                    TimeAgo = "4天前"
                },
                new NoteItem
                {
                    NoteContent = "健身计划：周一胸部训练、周二背部训练、周三休息、周四肩部训练、周五腿部训练、周六有氧运动、周日休息。每次训练45-60分钟。",
                    TimeAgo = "5天前"
                },
                new NoteItem
                {
                    NoteContent = "电影推荐：《肖申克的救赎》、《阿甘正传》、《泰坦尼克号》、《盗梦空间》、《星际穿越》、《楚门的世界》、《辛德勒的名单》、《美丽人生》。",
                    TimeAgo = "1周前"
                },
                new NoteItem
                {
                    NoteContent = "Python学习笔记：列表推导式、生成器、装饰器、上下文管理器、元类、异步编程等高级特性的学习总结。",
                    TimeAgo = "1周前"
                },
                new NoteItem
                {
                    NoteContent = "旅行攻略：日本东京5日游，第一天浅草寺、第二天秋叶原、第三天银座、第四天迪士尼、第五天自由活动。交通推荐购买Suica卡。",
                    TimeAgo = "2周前"
                }
            };

            NotesRepeater.ItemsSource = notes;
        }
    }
}
