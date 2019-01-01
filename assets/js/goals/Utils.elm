module Goals.Utils exposing (encode, end_date, goalDecoder, goalUpdatedDecoder, goalsDecoder, id, start_date, target, title)

import Goals.Models
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


goalsDecoder : Json.Decode.Decoder (List Goals.Models.Goal)
goalsDecoder =
    Json.Decode.field "data" (Json.Decode.list goalDecoder)


goalDecoder : Json.Decode.Decoder Goals.Models.Goal
goalDecoder =
    Json.Decode.succeed Goals.Models.Goal
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "start_date" Json.Decode.string
        |> Json.Decode.Pipeline.required "end_date" Json.Decode.string
        |> Json.Decode.Pipeline.required "target" Json.Decode.float


goalUpdatedDecoder : Json.Decode.Decoder Goals.Models.Goal
goalUpdatedDecoder =
    Json.Decode.field "data" goalDecoder


id : Int -> ( String, Json.Encode.Value )
id value =
    ( "id", Json.Encode.int value )


title : String -> ( String, Json.Encode.Value )
title value =
    ( "title", Json.Encode.string value )


end_date : String -> ( String, Json.Encode.Value )
end_date value =
    ( "end_date", Json.Encode.string value )


start_date : String -> ( String, Json.Encode.Value )
start_date value =
    ( "start_date", Json.Encode.string value )


target : Float -> ( String, Json.Encode.Value )
target value =
    ( "target", Json.Encode.float value )


encode : Goals.Models.Goal -> Json.Encode.Value
encode schema =
    Json.Encode.object
        [ ( "goal"
          , Json.Encode.object
                [ id schema.id
                , title schema.title
                , start_date schema.startDate
                , end_date schema.endDate
                , target schema.target
                ]
          )
        ]
