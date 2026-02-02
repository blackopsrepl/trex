use ratatui::style::Color;

// Glowing emerald green for selected agents
pub const EMERALD_GREEN: Color = Color::Rgb(80, 200, 120);

// The majestic T-Rex king, holding his tmux windows
pub const TREX_ASCII: &str = r#"                  \/
              \  _||_  /
               \/*||*\/
             .-=-==--==--.
       ..-=="  ,'o`)      `.
     ,'         `"'         \
    :  (                     `.__...._
    |                  )    /         `-=-.
    :       ,vv.-._   /    /               `---==-._
     \/\/\/VV ^ d88`;'    /                         `.
         ``  ^/d88P!'    /             ,              `._
            ^/    !'   ,.      ,      /                  "-,,__,,--'""""-.
           ^/    !'  ,'  \ . .(      (         _           )  ) ) ) ))_,-.\
          ^(__ ,!',"'   ;:+.:%:a.     \:.. . ,'          )  )  ) ) ,"'    '
          ',,,'','     /o:::":%:%a.    \:.:.:         .    )  ) _,'
           """'       ;':::'' `+%%%a._  \%:%|         ;.). _,-""
                  ,-='_.-'      ``:%::)  )%:|        /:._,"
                 (/(/"        .-----. ,'%%%:       (_,'
                             |  $   | ___;        \
                             | vim  |\   `         `
                              `-----' `.   `.        :
                        .-----.  \. . .\    : . . . :
                       | htop  \  \. . .:    `.. . .:
                       |  $$   |   `..:.:\     \:...\
                        `-----'     ;:.:.;      ::...:
                                    ):%::       :::::;
                                __,::%:(        :::::
                             ,;:%%%%%%%:        ;:%::
                               ;,--""-.`\  ,=--':%:%:\
                              /"       "| /-".:%%%%%%%\
                                              ;,-"'`)%%)
                                             /"      "|"#;

// The eye is on line 4 (0-indexed), character 'o'
pub const EYE_LINE: usize = 4;
pub const EYE_CHAR: char = 'o';

// Green rainbow gradient - from dark to lime to bright (lolcat-style)
pub const GREEN_GRADIENT: [(u8, u8, u8); 8] = [
    (0, 60, 20),    // Dark forest
    (0, 90, 30),    // Deep green
    (0, 120, 40),   // Forest green
    (20, 150, 50),  // Green
    (40, 180, 60),  // Bright green
    (80, 200, 80),  // Lime
    (40, 180, 60),  // Bright green (back down)
    (20, 150, 50),  // Green
];
