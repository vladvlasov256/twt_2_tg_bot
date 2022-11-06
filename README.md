# @twt_2_tg_bot
Telegram bot that zhuzh shared Twitter content

## Video converter

Telegram doesn't provide the ability to play Twitter videos inside the app. The bot converts them to regular MP4 files.

![Tweet with video link converted with bot](screenshots/video.gif)

## Image converter

The bot allows to download images from a tweet.

![Tweet with images link converted with bot](screenshots/image.gif)

## Text converter

> This feature is outdated because Telegram fixed their previews

Some tweets may contain line breaks or even dialogs.

![Tweet with several lines of text](screenshots/original_text.jpg)

Such tweets are barely readable in Telegram. The bot keeps original formatting.

![Tweet with several lines of text link converted with the bot](screenshots/text.gif)

## Dependencies

* [teloxide](https://github.com/teloxide/teloxide)
* [egg-mode](https://github.com/egg-mode-rs/egg-mode)