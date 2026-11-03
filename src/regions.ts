export type RegionConfig = {
  label: string;
  timezone: string;
  locale: string;
  language: string;
  browserLanguage: string;
  windowsTimezone: string;
};

export type RegionGroup = {
  label: string;
  options: RegionConfig[];
};

// These are common, supported Claude locations represented by a concrete city
// so the selected value can drive a deterministic timezone and locale change.
// Availability and account eligibility remain subject to Anthropic's current
// policy and the user's actual location.
export const REGION_GROUPS: RegionGroup[] = [
  {
    label: "北美",
    options: [
      { label: "美国 · 洛杉矶", timezone: "America/Los_Angeles", locale: "en_US", language: "en-US", browserLanguage: "en-US,en", windowsTimezone: "Pacific Standard Time" },
      { label: "美国 · 旧金山", timezone: "America/Los_Angeles", locale: "en_US", language: "en-US", browserLanguage: "en-US,en", windowsTimezone: "Pacific Standard Time" },
      { label: "美国 · 西雅图", timezone: "America/Los_Angeles", locale: "en_US", language: "en-US", browserLanguage: "en-US,en", windowsTimezone: "Pacific Standard Time" },
      { label: "美国 · 纽约", timezone: "America/New_York", locale: "en_US", language: "en-US", browserLanguage: "en-US,en", windowsTimezone: "Eastern Standard Time" },
      { label: "美国 · 芝加哥", timezone: "America/Chicago", locale: "en_US", language: "en-US", browserLanguage: "en-US,en", windowsTimezone: "Central Standard Time" },
      { label: "加拿大 · 多伦多", timezone: "America/Toronto", locale: "en_CA", language: "en-CA", browserLanguage: "en-CA,en-US,en", windowsTimezone: "Eastern Standard Time" },
      { label: "加拿大 · 温哥华", timezone: "America/Vancouver", locale: "en_CA", language: "en-CA", browserLanguage: "en-CA,en-US,en", windowsTimezone: "Pacific Standard Time" },
      { label: "墨西哥 · 墨西哥城", timezone: "America/Mexico_City", locale: "es_MX", language: "es-MX", browserLanguage: "es-MX,es,en-US,en", windowsTimezone: "Central Standard Time (Mexico)" },
    ],
  },
  {
    label: "南美",
    options: [
      { label: "巴西 · 圣保罗", timezone: "America/Sao_Paulo", locale: "pt_BR", language: "pt-BR", browserLanguage: "pt-BR,pt,en-US,en", windowsTimezone: "E. South America Standard Time" },
      { label: "阿根廷 · 布宜诺斯艾利斯", timezone: "America/Argentina/Buenos_Aires", locale: "es_AR", language: "es-AR", browserLanguage: "es-AR,es,en-US,en", windowsTimezone: "Argentina Standard Time" },
    ],
  },
  {
    label: "欧洲",
    options: [
      { label: "英国 · 伦敦", timezone: "Europe/London", locale: "en_GB", language: "en-GB", browserLanguage: "en-GB,en-US,en", windowsTimezone: "GMT Standard Time" },
      { label: "爱尔兰 · 都柏林", timezone: "Europe/Dublin", locale: "en_IE", language: "en-IE", browserLanguage: "en-IE,en-GB,en-US,en", windowsTimezone: "GMT Standard Time" },
      { label: "德国 · 法兰克福", timezone: "Europe/Berlin", locale: "de_DE", language: "de-DE", browserLanguage: "de-DE,de,en-US,en", windowsTimezone: "W. Europe Standard Time" },
      { label: "法国 · 巴黎", timezone: "Europe/Paris", locale: "fr_FR", language: "fr-FR", browserLanguage: "fr-FR,fr,en-US,en", windowsTimezone: "Romance Standard Time" },
      { label: "荷兰 · 阿姆斯特丹", timezone: "Europe/Amsterdam", locale: "nl_NL", language: "nl-NL", browserLanguage: "nl-NL,nl,en-US,en", windowsTimezone: "W. Europe Standard Time" },
      { label: "瑞士 · 苏黎世", timezone: "Europe/Zurich", locale: "de_CH", language: "de-CH", browserLanguage: "de-CH,de,en-US,en", windowsTimezone: "W. Europe Standard Time" },
      { label: "西班牙 · 马德里", timezone: "Europe/Madrid", locale: "es_ES", language: "es-ES", browserLanguage: "es-ES,es,en-US,en", windowsTimezone: "Romance Standard Time" },
      { label: "意大利 · 米兰", timezone: "Europe/Rome", locale: "it_IT", language: "it-IT", browserLanguage: "it-IT,it,en-US,en", windowsTimezone: "W. Europe Standard Time" },
      { label: "葡萄牙 · 里斯本", timezone: "Europe/Lisbon", locale: "pt_PT", language: "pt-PT", browserLanguage: "pt-PT,pt,en-US,en", windowsTimezone: "GMT Standard Time" },
      { label: "波兰 · 华沙", timezone: "Europe/Warsaw", locale: "pl_PL", language: "pl-PL", browserLanguage: "pl-PL,pl,en-US,en", windowsTimezone: "Central European Standard Time" },
      { label: "瑞典 · 斯德哥尔摩", timezone: "Europe/Stockholm", locale: "sv_SE", language: "sv-SE", browserLanguage: "sv-SE,sv,en-US,en", windowsTimezone: "W. Europe Standard Time" },
    ],
  },
  {
    label: "亚洲与大洋洲",
    options: [
      { label: "日本 · 东京", timezone: "Asia/Tokyo", locale: "ja_JP", language: "ja-JP", browserLanguage: "ja-JP,ja,en-US,en", windowsTimezone: "Tokyo Standard Time" },
      { label: "韩国 · 首尔", timezone: "Asia/Seoul", locale: "ko_KR", language: "ko-KR", browserLanguage: "ko-KR,ko,en-US,en", windowsTimezone: "Korea Standard Time" },
      { label: "新加坡", timezone: "Asia/Singapore", locale: "en_SG", language: "en-SG", browserLanguage: "en-SG,en-US,en", windowsTimezone: "Singapore Standard Time" },
      { label: "台湾 · 台北", timezone: "Asia/Taipei", locale: "zh_TW", language: "zh-TW", browserLanguage: "zh-TW,zh,en-US,en", windowsTimezone: "Taipei Standard Time" },
      { label: "澳大利亚 · 悉尼", timezone: "Australia/Sydney", locale: "en_AU", language: "en-AU", browserLanguage: "en-AU,en-US,en", windowsTimezone: "AUS Eastern Standard Time" },
      { label: "澳大利亚 · 墨尔本", timezone: "Australia/Melbourne", locale: "en_AU", language: "en-AU", browserLanguage: "en-AU,en-US,en", windowsTimezone: "AUS Eastern Standard Time" },
      { label: "新西兰 · 奥克兰", timezone: "Pacific/Auckland", locale: "en_NZ", language: "en-NZ", browserLanguage: "en-NZ,en-AU,en-US,en", windowsTimezone: "New Zealand Standard Time" },
      { label: "印度 · 孟买", timezone: "Asia/Kolkata", locale: "en_IN", language: "en-IN", browserLanguage: "en-IN,en-US,en", windowsTimezone: "India Standard Time" },
      { label: "马来西亚 · 吉隆坡", timezone: "Asia/Kuala_Lumpur", locale: "ms_MY", language: "ms-MY", browserLanguage: "ms-MY,ms,en-US,en", windowsTimezone: "Singapore Standard Time" },
      { label: "泰国 · 曼谷", timezone: "Asia/Bangkok", locale: "th_TH", language: "th-TH", browserLanguage: "th-TH,th,en-US,en", windowsTimezone: "SE Asia Standard Time" },
      { label: "越南 · 胡志明市", timezone: "Asia/Ho_Chi_Minh", locale: "vi_VN", language: "vi-VN", browserLanguage: "vi-VN,vi,en-US,en", windowsTimezone: "SE Asia Standard Time" },
      { label: "菲律宾 · 马尼拉", timezone: "Asia/Manila", locale: "en_PH", language: "en-PH", browserLanguage: "en-PH,en-US,en", windowsTimezone: "Singapore Standard Time" },
      { label: "印度尼西亚 · 雅加达", timezone: "Asia/Jakarta", locale: "id_ID", language: "id-ID", browserLanguage: "id-ID,id,en-US,en", windowsTimezone: "SE Asia Standard Time" },
      { label: "以色列 · 特拉维夫", timezone: "Asia/Jerusalem", locale: "he_IL", language: "he-IL", browserLanguage: "he-IL,he,en-US,en", windowsTimezone: "Israel Standard Time" },
      { label: "阿联酋 · 迪拜", timezone: "Asia/Dubai", locale: "ar_AE", language: "ar-AE", browserLanguage: "ar-AE,ar,en-US,en", windowsTimezone: "Arabian Standard Time" },
    ],
  },
  {
    label: "非洲",
    options: [
      { label: "南非 · 约翰内斯堡", timezone: "Africa/Johannesburg", locale: "en_ZA", language: "en-ZA", browserLanguage: "en-ZA,en-US,en", windowsTimezone: "South Africa Standard Time" },
    ],
  },
];

export const REGION_OPTIONS = REGION_GROUPS.flatMap((group) => group.options);
export const DEFAULT_REGION = "美国 · 洛杉矶";

export function getRegionConfig(targetRegion: string): RegionConfig {
  return REGION_OPTIONS.find((option) => option.label === targetRegion) ?? REGION_OPTIONS[0];
}
